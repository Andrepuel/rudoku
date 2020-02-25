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
    fn value(&mut self) -> Rc<dyn Value<T>>;
}

trait Value<T> {
    fn set_observer(&self, observer: Weak<dyn Observer>);
    fn get(&self) -> T;
}
struct StateValueValue<T> {
    value: RefCell<T>,
    observer: RefCell<Weak<dyn Observer>>,
}
impl<T: Copy> Value<T> for StateValueValue<T> {
    fn set_observer(&self, observer: Weak<dyn Observer>) {
        *self.observer.borrow_mut() = observer;
    }

    fn get(&self) -> T {
        *self.value.borrow()
    }
}
struct StateValue<T> {
    observers: Vec<Rc<StateValueValue<T>>>,
    value: T,
}
impl<T: Copy> StateValue<T> {
    fn new(initial: T) -> StateValue<T> {
        StateValue {
            observers: Vec::new(),
            value: initial,
        }
    }

    fn lock_observers(&mut self) -> Vec<(Rc<dyn Observer>, Rc<StateValueValue<T>>)> {
        let mut i = 0;
        let mut r = Vec::new();
        while i < self.observers.len() {
            if let Some(lock) = self.observers[i].clone().observer.borrow().upgrade() {
                r.push((lock, self.observers[i].clone()));
                i += 1;
            } else {
                self.observers.remove(i);
            };
        };

        r
    }

    fn set(&mut self, value: T) {
        self.value = value;
        self.lock_observers()
        .into_iter()
        .for_each(|x| {
            *x.1.value.borrow_mut() = value;
            x.0.update();
        });
    }
}
impl<T: Copy> Observed<T> for StateValue<T> where T: 'static {
    fn value(&mut self) -> Rc<dyn Value<T>> {
        let r = Rc::new(StateValueValue {
            value: RefCell::new(self.value),
            observer: RefCell::new(Weak::<EmptyObserver>::new()),
        });
        self.observers.push(r.clone());
        r
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
    fn value(&mut self) -> Rc<dyn Value<SystemTime>> {
        self.value.value()
    }
}

struct Printer<T> {
    value: Rc<dyn Value<T>>,
}
impl<T: std::fmt::Debug> Printer<T> where T: 'static {
    fn new(observed: &mut dyn Observed<T>) -> Rc<Printer<T>> {
        let value = observed.value();
        let r = Rc::new(Printer::<T> {
            value: value.clone(),
        });
        let observer: Rc<dyn Observer> = r.clone();
        value.set_observer(Rc::downgrade(&observer));
        r
    }
}
impl<T: std::fmt::Debug> Observer for Printer<T> {
    fn update(&self) {
        println!("Got {:?}", self.value.get());
    }
}

fn main() {
    let mut clock = ClockValue::new();
    let x = Printer::new(&mut clock);
    clock.run();
}
