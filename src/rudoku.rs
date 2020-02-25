use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait Observer {
    fn update(&self);
}
pub trait ChildObserved<T, U> {
    fn value(&mut self, observer: &Rc<dyn Observer>, input: Box<dyn Value<U>>)
        -> Box<dyn Value<T>>;
}
struct ChildObservedFuse<T, U> {
    underlying: Box<dyn ChildObserved<T, U>>,
    input: Box<dyn Observed<U>>,
}
impl<T, U> Observed<T> for ChildObservedFuse<T, U> {
    fn value(&mut self, observer: &Rc<dyn Observer>) -> Box<dyn Value<T>> {
        self.underlying.value(observer, self.input.value(observer))
    }
}
pub trait ChildObservedExt<T, U> {
    fn fuse(self: Box<Self>, input: Box<dyn Observed<U>>) -> Box<dyn Observed<T>>;
}
impl<T, U> ChildObservedExt<T, U> for dyn ChildObserved<T, U>
where
    T: 'static,
    U: 'static,
{
    fn fuse(self: Box<Self>, input: Box<dyn Observed<U>>) -> Box<dyn Observed<T>> {
        Box::new(ChildObservedFuse::<T, U> {
            underlying: self,
            input,
        })
    }
}
pub trait Observed<T> {
    fn value(&mut self, observer: &Rc<dyn Observer>) -> Box<dyn Value<T>>;
}
pub trait Value<T> {
    fn get(&self) -> T;
}
struct ValueMap<T, U, F: Fn(T) -> U> {
    underlying: Box<dyn Value<T>>,
    adapter: F,
}
impl<T, U, F: Fn(T) -> U> Value<U> for ValueMap<T, U, F>
where
    T: 'static,
    U: 'static,
{
    fn get(&self) -> U {
        (self.adapter)(self.underlying.get())
    }
}
struct ValueJoin<T, U> {
    underlying: Box<dyn Value<T>>,
    other: Box<dyn Value<U>>,
}
impl<T, U> Value<(T, U)> for ValueJoin<T, U>
where
    T: 'static,
    U: 'static,
{
    fn get(&self) -> (T, U) {
        (self.underlying.get(), self.other.get())
    }
}
pub struct ValueSplit<T> {
    rc: Rc<Box<dyn Value<T>>>,
}
impl<T> Value<T> for ValueSplit<T> {
    fn get(&self) -> T {
        self.rc.get()
    }
}
impl<T> ValueSplit<T>
where
    T: 'static,
{
    pub fn take(&self) -> Box<dyn Value<T>> {
        Box::new(ValueSplit {
            rc: self.rc.clone(),
        })
    }
}
pub trait ValueExt<T> {
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>>
    where
        U: 'static,
        F: 'static;
    fn join<U>(self: Box<Self>, other: Box<dyn Value<U>>) -> Box<dyn Value<(T, U)>>
    where
        U: 'static;
    fn split(self: Box<Self>) -> ValueSplit<T>;
}
impl<T> ValueExt<T> for dyn Value<T>
where
    T: 'static,
{
    fn map<U, F: Fn(T) -> U>(self: Box<Self>, adapter: F) -> Box<dyn Value<U>>
    where
        U: 'static,
        F: 'static,
    {
        Box::new(ValueMap::<T, U, F> {
            underlying: self,
            adapter,
        })
    }

    fn join<U>(self: Box<Self>, other: Box<dyn Value<U>>) -> Box<dyn Value<(T, U)>>
    where
        U: 'static,
    {
        Box::new(ValueJoin::<T, U> {
            underlying: self,
            other: other,
        })
    }

    fn split(self: Box<Self>) -> ValueSplit<T> {
        let rc = Rc::new(self);

        ValueSplit::<T> { rc }
    }
}

struct StateValueValue<T> {
    value: Rc<RefCell<T>>,
}

struct StateValueObserver<T> {
    observers: Weak<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> Observed<T> for StateValueObserver<T>
where
    T: 'static,
{
    fn value(&mut self, observer: &Rc<dyn Observer>) -> Box<dyn Value<T>> {
        if let Some(observers) = self.observers.upgrade() {
            observers.borrow_mut().push(Rc::downgrade(observer));
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
pub struct StateValue<T> {
    observers: Rc<RefCell<Vec<Weak<dyn Observer>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> StateValue<T>
where
    T: 'static,
{
    pub fn new(initial: T) -> (StateValue<T>, Box<dyn Observed<T>>) {
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
        }

        r
    }

    pub fn set(&mut self, value: T) {
        *self.value.borrow_mut() = value;
        self.lock_observers().into_iter().for_each(|x| x.update());
    }
}
