use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn update(&self);
}

trait Observed<T> {
    fn observe(&self, observe: Rc<dyn Observer>) -> Rc<dyn Value<T>>;
}

trait Value<T> {
    fn get(&self) -> T;
}

struct StateValue<T> {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
    value: Rc<StateValueValue<T>>,
}
impl<T> StateValue<T> {
    fn new(initial: T) -> StateValue<T> {
        StateValue {
            observers: RefCell::new(Vec::new()),
            value: Rc::new(StateValueValue::<T>{value: RefCell::new(initial)}),
        }
    }

    fn lock_observers(&self) -> Vec<Rc<dyn Observer>> {
        let mut i = 0;
        let mut r = Vec::new();
        let mut observers = self.observers.borrow_mut();
        while i < observers.len() {
            if let Some(lock) = observers[i].upgrade() {
                r.push(lock);
                i += 1;
            } else {
                observers.remove(i);
            }
        }

        r
    }

    fn set(&self, value: T) {
        *self.value.value.borrow_mut() = value;
        self.lock_observers()
        .into_iter()
        .for_each(|x| x.update());
    }
}
struct StateValueValue<T> {
    value: RefCell<T>,
}
impl<T: Copy> Value<T> for StateValueValue<T> {
    fn get(&self) -> T {
        *self.value.borrow()
    }
}
impl<T: Clone + Copy> Observed<T> for StateValue<T> where T : 'static {
    fn observe(&self, observe: Rc<dyn Observer>) -> Rc<dyn Value<T>> {
        self.observers.borrow_mut().push(Rc::downgrade(&observe));
        self.value.clone()
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

    fn run(&self) {
        loop {
            sleep(Duration::new(1, 0));
            self.value.set(SystemTime::now());
        }
    }
}
impl Observed<SystemTime> for ClockValue {
    fn observe(&self, observe: Rc<dyn Observer>) -> Rc<dyn Value<SystemTime>> {
        self.value.observe(observe)
    }
}

struct X {
    clock: RefCell<Option<Rc<dyn Value<SystemTime>>>>,
}
impl X {
    fn new(clock: &ClockValue) -> Rc<X> {
        let r = X{clock: RefCell::new(None)};
        let r = Rc::new(r);
        *r.clock.borrow_mut() = Some(clock.observe(r.clone()));
        r
    }
}
impl Observer for X {
    fn update(&self) {
        println!("Clock {:?}", self.clock.borrow().as_ref().unwrap().get());
    }
}

fn main() {
    let clock = ClockValue::new();
    let x = X::new(&clock);
    clock.run();
}
