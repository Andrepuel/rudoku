use rudoku::rudoku::{
    ChildObserved, ChildObservedExt, Observed, Observer, StateValue, Value, ValueExt,
};
use rudoku::text::Decorated;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

struct ClockValue {
    value: StateValue<SystemTime>,
}
impl ClockValue {
    fn new() -> (ClockValue, Box<dyn Observed<SystemTime>>) {
        let (value, observed) = StateValue::new(SystemTime::now());

        (ClockValue { value }, observed)
    }

    fn run(&mut self) {
        loop {
            sleep(Duration::new(1, 0));
            self.value.set(SystemTime::now());
        }
    }
}

struct Printer<T> {
    value: RefCell<Option<Box<dyn Value<T>>>>,
}
impl<T: std::fmt::Display> Printer<T>
where
    T: 'static,
{
    fn new(mut observed: Box<dyn Observed<T>>) -> Rc<Printer<T>> {
        let r = Rc::new(Printer::<T> {
            value: RefCell::new(None),
        });
        let observer: Rc<dyn Observer> = r.clone();
        let value = observed.value(&observer);
        *r.value.borrow_mut() = Some(value);
        r
    }
}
impl<T: std::fmt::Display> Observer for Printer<T> {
    fn update(&self) {
        println!("{}", self.value.borrow().as_ref().unwrap().get());
    }
}

struct PrettyClock {}
impl PrettyClock {
    fn new() -> PrettyClock {
        PrettyClock {}
    }
}
impl ChildObserved<String, SystemTime> for PrettyClock {
    fn value(
        &mut self,
        _: &Rc<dyn Observer>,
        input: Box<dyn Value<SystemTime>>,
    ) -> Box<dyn Value<String>> {
        input.map(|x| match x.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        })
    }
}

struct Main {
    text: PrettyClock,
    decorated: Decorated,
}
impl Main {
    fn new() -> Main {
        Main {
            text: PrettyClock::new(),
            decorated: Decorated::new(),
        }
    }
}
impl ChildObserved<String, SystemTime> for Main {
    fn value(
        &mut self,
        observer: &Rc<dyn Observer>,
        input: Box<dyn Value<SystemTime>>,
    ) -> Box<dyn Value<String>> {
        self.decorated.value(
            observer,
            self.text.value(observer, input).map(|x| (3, 3, x)),
        )
    }
}

fn main() {
    let (mut runner, clock) = ClockValue::new();

    let main: Box<dyn ChildObserved<String, SystemTime>> = Box::new(Main::new());
    let _output = Printer::new(main.fuse(clock));
    runner.run();
}
