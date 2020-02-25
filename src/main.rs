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
    fn value(&mut self) -> Box<dyn Value<T>>;
}

trait Value<T> {
    fn set_observer(&mut self, observer: Weak<dyn Observer>);
    fn get(&self) -> T;
}


struct StateValueValue<T> {
    observers: Weak<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}

impl<T: Copy> Value<T> for StateValueValue<T> {
    fn set_observer(&mut self, observer: Weak<dyn Observer>) {
        if let Some(observers) = self.observers.upgrade() {
            observers.borrow_mut().push(observer);
        }
    }

    fn get(&self) -> T {
        *self.value.borrow()
    }
}
struct StateValue<T> {
    observers: Rc<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> StateValue<T> {
    fn new(initial: T) -> StateValue<T> {
        StateValue {
            observers: Rc::new(RefCell::new(Vec::new())),
            value: Rc::new(RefCell::new(initial)),
        }
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
impl<T: Copy> Observed<T> for StateValue<T> where T: 'static {
    fn value(&mut self) -> Box<dyn Value<T>> {
        Box::new(StateValueValue {
            observers: Rc::downgrade(&self.observers),
            value: self.value.clone(),
        })
    }
}

use std::time::{Duration, SystemTime};
use std::thread::sleep;

struct ClockValue {
    value: StateValue<SystemTime>,
}
impl ClockValue {
    fn new() -> ClockValue {
        ClockValue {
            value: StateValue::new(SystemTime::now()),
        }
    }

    fn run(&mut self) {
        loop {
            sleep(Duration::new(1, 0));
            self.value.set(SystemTime::now());
        }
    }
}
impl Observed<SystemTime> for ClockValue {
    fn value(&mut self) -> Box<dyn Value<SystemTime>> {
        self.value.value()
    }
}

struct Printer<T> {
    value: RefCell<Box<dyn Value<T>>>,
}
impl<T: std::fmt::Debug> Printer<T> where T: 'static {
    fn new(observed: &mut dyn Observed<T>) -> Rc<Printer<T>> {
        let value = observed.value();
        let r = Rc::new(Printer::<T> {
            value: RefCell::new(value),
        });
        let observer: Rc<dyn Observer> = r.clone();
        r.value.borrow_mut().set_observer(Rc::downgrade(&observer));
        r
    }
}
impl<T: std::fmt::Debug> Observer for Printer<T> {
    fn update(&self) {
        println!("Got {:?}", self.value.borrow().get());
    }
}

fn main() {
    let mut clock = ClockValue::new();
    let x = Printer::new(&mut clock);
    clock.run();
}
