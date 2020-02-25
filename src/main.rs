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
trait ChildObserved<T, U> {
    fn value(&mut self, observer: Weak<dyn Observer>, input: Box<dyn Value<U>>) -> Box<dyn Value<T>>;
}
struct ChildObservedFuse<T, U> {
    underlying: Box<dyn ChildObserved<T, U>>,
    input: Box<dyn Observed<U>>,
}
impl<T,U> Observed<T> for ChildObservedFuse<T, U> {
    fn value(&mut self, observer: Weak<dyn Observer>) -> Box<dyn Value<T>> {
        self.underlying.value(observer.clone(), self.input.value(observer.clone()))
    }
}
trait ChildObservedExt<T, U> {
    fn fuse(self: Box<Self>, input: Box<dyn Observed<U>>) -> Box<dyn Observed<T>>;
}
impl<T, U> ChildObservedExt<T, U> for dyn ChildObserved<T, U> where T: 'static, U: 'static {
    fn fuse(self: Box<Self>, input: Box<dyn Observed<U>>) -> Box<dyn Observed<T>> {
        Box::new(ChildObservedFuse::<T,U> {
            underlying: self,
            input
        })
    }
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
impl<T, U, F: Fn(T) -> U> Value<U> for ValueMap<T, U, F> where T: 'static, U: 'static {
    fn get(&self) -> U {
        (self.adapter)(self.underlying.get())
    }
}
struct ValueJoin<T, U> {
    underlying: Box<dyn Value<T>>,
    other: Box<dyn Value<U>>,
}
impl<T, U> Value<(T,U)> for ValueJoin<T,U> where T: 'static, U: 'static {
    fn get(&self) -> (T,U) {
        (self.underlying.get(), self.other.get())
    }
}
struct ValueSplit<T> {
    rc: Rc<Box<dyn Value<T>>>,
}
impl<T> Value<T> for ValueSplit<T> {
    fn get(&self) -> T {
        self.rc.get()
    }
}
impl<T> ValueSplit<T> where T: 'static {
    fn take(&self) -> Box<dyn Value<T>> {
        Box::new(ValueSplit {
            rc: self.rc.clone()
        })
    }
}
trait ValueExt<T> {
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>> where U: 'static, F: 'static;
    fn join<U>(self: Box<Self>, other: Box<dyn Value<U>>) -> Box<dyn Value<(T,U)>> where U: 'static;
    fn split(self: Box<Self>) -> ValueSplit<T>;
}
impl<T> ValueExt<T> for dyn Value<T> where T: 'static {
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>> where U: 'static, F: 'static {
        Box::new(ValueMap::<T, U, F> {
            underlying: self,
            adapter
        })
    }

    fn join<U>(self: Box<Self>, other: Box<dyn Value<U>>) -> Box<dyn Value<(T,U)>> where U: 'static {
        Box::new(ValueJoin::<T, U> {
            underlying: self,
            other: other,
        })
    }

    fn split(self: Box<Self>) -> ValueSplit<T> {
        let rc = Rc::new(self);

        ValueSplit::<T> {
            rc
        }
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
impl<T: std::fmt::Display> Printer<T> where T: 'static {
    fn new(mut observed: Box<dyn Observed<T>>) -> Rc<Printer<T>> {
        let r = Rc::new(Printer::<T> { value: RefCell::new(None)});
        let observer: Rc<dyn Observer> = r.clone();
        let value = observed.value(Rc::downgrade(&observer));
        *r.value.borrow_mut() = Some(value);
        r
    }
}
impl<T: std::fmt::Display> Observer for Printer<T> {
    fn update(&self) {
        println!("{}", self.value.borrow().as_ref().unwrap().get());
    }
}

struct PrettyClock {
}
impl PrettyClock {
    fn new() -> PrettyClock {
        PrettyClock {}
    }
}
impl ChildObserved<String, SystemTime> for PrettyClock {
    fn value(&mut self, _: Weak<dyn Observer>, input: Box<dyn Value<SystemTime>>) -> Box<dyn Value<String>> {
        input
        .map(|x| match x.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        })
    }
}

struct PositionedText {
}
impl PositionedText {
    fn new() -> PositionedText {
        PositionedText {}
    }

    fn position(row: i32, column: i32) -> String {
        format!("\x1b[{};{}H", row, column)
    }
}
impl ChildObserved<String, (i32, i32, String)> for PositionedText {
    fn value(&mut self, _: Weak<dyn Observer>, input: Box<dyn Value<(i32, i32, String)>>) -> Box<dyn Value<String>> {
        ValueExt::map(input,
        |x| format!("{}{}", PositionedText::position(x.0, x.1), x.2))
    }
}

struct HorizontalLine {
    positioned: PositionedText,
}
impl HorizontalLine {
    fn new() -> HorizontalLine {
        HorizontalLine {
            positioned: PositionedText::new()
        }
    }

    fn line(length: i32) -> String {
        std::iter::repeat("=").take(length as usize).collect::<String>()
    }
}
impl ChildObserved<String, (i32, i32, i32)> for HorizontalLine {
    fn value(&mut self, observer: Weak<dyn Observer>, input: Box<dyn Value<(i32, i32, i32)>>) -> Box<dyn Value<String>> {
        self.positioned.value(observer,
            input
            .map(|x| (x.0, x.1, HorizontalLine::line(x.2)))
        )
    }
}

struct VerticalLine {
}
impl VerticalLine {
    fn new() -> VerticalLine {
        VerticalLine {}
    }
}
impl ChildObserved<String, (i32, i32, i32)> for VerticalLine {
    fn value(&mut self, _: Weak<dyn Observer>, input: Box<dyn Value<(i32, i32, i32)>>) -> Box<dyn Value<String>> {
        input
        .map(|(row,col,len)| {
            let mut r = String::new();
            for i in 0..len {
                r.push_str(&format!("{}|", PositionedText::position(row + i, col)));
            }
            r
        })
    }
}

struct Clear {
}
impl Clear {
    fn new() -> Clear {
        Clear {}
    }

    fn clear(length: i32) -> String {
        std::iter::repeat(" ").take(length as usize).collect::<String>()
    }
}
impl ChildObserved<String, (i32, i32, i32, i32)> for Clear {
    fn value(&mut self, _: Weak<dyn Observer>, input: Box<dyn Value<(i32, i32, i32, i32)>>) -> Box<dyn Value<String>> {
        input
        .map(|(row, col, width, height)| {
            let mut r = String::new();
            for i in 0..height {
                r.push_str(&format!("{}{}", PositionedText::position(row + i, col), Clear::clear(width)))
            }
            r
        })
    }
}

struct Border {
    top : HorizontalLine,
    bottom : HorizontalLine,
    left : VerticalLine,
    right: VerticalLine,
}
impl Border {
    fn new() -> Border {
        Border {
            top: HorizontalLine::new(),
            bottom: HorizontalLine::new(),
            left: VerticalLine::new(),
            right: VerticalLine::new(),
        }
    }
}
impl ChildObserved<String, (i32, i32, i32, i32)> for Border {
    fn value(&mut self, observer: Weak<dyn Observer>, input_unique: Box<dyn Value<(i32, i32, i32, i32)>>) -> Box<dyn Value<String>> {
        let input = input_unique.split();

        self.top.value(observer.clone(),
            input.take()
            .map(|(row, col, width, _height)| (row, col, width))
        ).join(self.bottom.value(observer.clone(),
            input.take()
            .map(|(row, col, width, height)| (row + height - 1, col, width))
        )).map(|x| format!("{}{}", x.0, x.1))
        .join(self.left.value(observer.clone(),
            input.take()
            .map(|(row, col, _width, height)| (row, col, height))
        )).map(|x| format!("{}{}", x.0, x.1))
        .join(self.right.value(observer.clone(),
            input.take()
            .map(|(row, col, width, height)| (row, col + width - 1, height))
        )).map(|x| format!("{}{}", x.0, x.1))
    }
}

struct Decorated {
    clear: Clear,
    border: Border,
    positioned: PositionedText,
}
impl Decorated {
    fn new() -> Decorated {
        Decorated {
            clear: Clear::new(),
            border: Border::new(),
            positioned: PositionedText::new(),
        }
    }
}
impl ChildObserved<String, (i32, i32, String)> for Decorated {
    fn value(&mut self, observer: Weak<dyn Observer>, input_unique: Box<dyn Value<(i32, i32, String)>>) -> Box<dyn Value<String>> {
        let input = input_unique.split();

        self.clear.value(observer.clone(),
            input.take()
            .map(|(row, col, text)| (row, col, (text.len() + 3) as i32, 5))
        ).join(self.border.value(observer.clone(),
            input.take()
            .map(|(row, col, text)| (row, col, (text.len() + 3) as i32, 5))
        )).map(|x| format!("{}{}", x.0, x.1))
        .join(self.positioned.value(observer.clone(),
            input.take()
            .map(|(row, col, text)| (row + 2, col + 2, text))
        )).map(|x| format!("{}{}", x.0, x.1))
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
    fn value(&mut self, observer: Weak<dyn Observer>, input: Box<dyn Value<SystemTime>>) -> Box<dyn Value<String>> {
        self.decorated.value(observer.clone(),
            self.text.value(observer.clone(), input)
            .map(|x| (3, 3, x))
        )
    }
}

fn main() {
    let (mut runner, clock) = ClockValue::new();

    let main: Box<dyn ChildObserved<String, SystemTime>> = Box::new(Main::new());
    let _output = Printer::new(main.fuse(clock));
    runner.run();
}
