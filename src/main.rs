use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn update(&self);
}
struct EmptyObserver {
}
impl Observer for EmptyObserver {
    fn update(&self) {}
}
trait Observed<T> {
    fn value(&mut self, observer: Weak<dyn Observer>) -> Box<dyn Value<T>>;
}

trait Value<T> {
    fn get(&self) -> T;
}
struct ValueMap<T, U, F: Fn(T) -> U> {
    underlying: Box<dyn Value<T>>,
    adapter: F,
}
impl<T: Copy, U, F: Fn(T) -> U> Value<U> for ValueMap<T, U, F> where T: 'static, U: 'static {
    fn get(&self) -> U {
        (self.adapter)(self.underlying.get())
    }
}
trait ValueExt<T> {
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>> where U: 'static, F: 'static;
}
impl<T: Copy> ValueExt<T> for dyn Value<T> where T: 'static {
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>> where U: 'static, F: 'static {
        Box::new(ValueMap::<T, U, F> {
            underlying: self,
            adapter
        })
    }
}

struct StateValueValue<T> {
    value: Rc<RefCell<T>>,
}

struct StateValueObserver<T> {
    observers: Weak<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> Observed<T> for StateValueObserver<T> where T: 'static {
    fn value(&mut self, observer: Weak<dyn Observer>) -> Box<dyn Value<T>> {
        if let Some(observers) = self.observers.upgrade() {
            observers.borrow_mut().push(observer);
        }

        Box::new(StateValueValue::<T> {
            value: self.value.clone(),
        })
    }
}

impl<T: Copy> Value<T> for StateValueValue<T> {
    fn get(&self) -> T {
        *self.value.borrow()
    }
}
struct StateValue<T> {
    observers: Rc<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> StateValue<T> where T: 'static {
    fn new(initial: T) -> (StateValue<T>, Box<dyn Observed<T>>) {
        let setter = StateValue {
            observers: Rc::new(RefCell::new(Vec::new())),
            value: Rc::new(RefCell::new(initial)),
        };

        let getter = StateValueObserver {
            observers: Rc::downgrade(&setter.observers),
            value: setter.value.clone(),
        };

        (setter, Box::new(getter))
    }

    fn lock_observers(&mut self) -> Vec<Rc<dyn Observer>> {
        let mut i = 0;
        let mut r = Vec::new();
        let mut observers = self.observers.borrow_mut();
        while i < observers.len() {
            if let Some(lock) = observers[i].upgrade() {
                r.push(lock);
                i += 1;
            } else {
                observers.remove(i);
            };
        };

        r
    }

    fn set(&mut self, value: T) {
        *self.value.borrow_mut() = value;
        self.lock_observers()
        .into_iter()
        .for_each(|x| x.update());
    }
}

use std::time::{Duration, SystemTime};
use std::thread::sleep;

struct ClockValue {
    value: StateValue<SystemTime>,
}
impl ClockValue {
    fn new() -> (ClockValue, Box<dyn Observed<SystemTime>>) {
        let (value, observed) = StateValue::new(SystemTime::now());

        (
            ClockValue {
                value,
            },
            observed
        )
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
impl<T: std::fmt::Debug> Printer<T> where T: 'static {
    fn new(observed: &mut dyn Observed<T>) -> Rc<Printer<T>> {
        let r = Rc::new(Printer::<T> { value: RefCell::new(None)});
        let observer: Rc<dyn Observer> = r.clone();
        let value = observed.value(Rc::downgrade(&observer));
        *r.value.borrow_mut() = Some(value);
        r
    }
}
impl<T: std::fmt::Debug> Observer for Printer<T> {
    fn update(&self) {
        println!("Got {:?}", self.value.borrow().as_ref().unwrap().get());
    }
}

struct PrettyClock {
    clock: Box<dyn Observed<SystemTime>>,
}
impl Observed<String> for PrettyClock {
    fn value(&mut self, observer: Weak<dyn Observer>) -> Box<dyn Value<String>> {
        self.clock.value(observer)
        .map(|x| match x.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        })
    }
}

fn main() {
    let (mut runner, clock) = ClockValue::new();
    let mut pretty = PrettyClock {
        clock,
    };
    let _x = Printer::new(&mut pretty);
    runner.run();
}
