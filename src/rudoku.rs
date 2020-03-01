use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait Observer {
    fn update(&self);
}
pub trait Observed<T> {
    fn value(&self, observer: &Rc<dyn Observer>) -> Rc<dyn Value<T>>;
}
pub fn fixed<T: Copy>(t: T) -> Box<dyn Observed<T>>
where
    T: 'static,
{
    struct FixedValue<T2> {
        value: T2,
    }
    impl<T2: Copy> Value<T2> for FixedValue<T2> {
        fn get(&self) -> T2 {
            self.value
        }
    }
    struct FixedObserved<T2> {
        value: Rc<FixedValue<T2>>,
    }
    impl<T2: Copy> Observed<T2> for FixedObserved<T2>
    where
        T2: 'static,
    {
        fn value(&self, _: &Rc<dyn Observer>) -> Rc<dyn Value<T2>> {
            self.value.clone()
        }
    }

    Box::new(FixedObserved::<T> {
        value: Rc::new(FixedValue::<T> { value: t }),
    })
}
#[derive(Clone)]
pub struct Split<T> {
    observed: Rc<dyn Observed<T>>,
}
impl<T: Clone> Split<T>
where
    T: 'static,
{
    pub fn take(&self) -> Box<dyn Observed<T>> {
        impl<T2> Observed<T2> for Split<T2> {
            fn value(&self, observer: &Rc<dyn Observer>) -> Rc<dyn Value<T2>> {
                self.observed.value(observer)
            }
        }

        Box::new(self.clone())
    }
}
pub trait ObservedExt<T> {
    fn map<U, F: Fn(T) -> U + Clone>(self: Box<Self>, f: F) -> Box<dyn Observed<U>>
    where
        F: 'static,
        U: 'static,
        T: 'static;
    fn join<U>(self: Box<Self>, other: Box<dyn Observed<U>>) -> Box<dyn Observed<(T, U)>>
    where
        U: 'static;
    fn split(self: Box<Self>) -> Split<T>;
}
impl<T> ObservedExt<T> for dyn Observed<T>
where
    T: 'static,
{
    fn map<U, F: Fn(T) -> U + Clone>(self: Box<Self>, f: F) -> Box<dyn Observed<U>>
    where
        F: 'static,
        U: 'static,
    {
        let r = (self, f);

        impl<T2, U2, F2: Fn(T2) -> U2 + Clone> Observed<U2> for (Box<dyn Observed<T2>>, F2)
        where
            F2: 'static,
            T2: 'static,
            U2: 'static,
        {
            fn value(&self, observer: &Rc<dyn Observer>) -> Rc<dyn Value<U2>> {
                self.0.value(observer).map(self.1.clone())
            }
        }

        Box::new(r)
    }

    fn join<U>(self: Box<Self>, other: Box<dyn Observed<U>>) -> Box<dyn Observed<(T, U)>>
    where
        U: 'static,
    {
        let r = (self, other);

        impl<T2, U2> Observed<(T2, U2)> for (Box<dyn Observed<T2>>, Box<dyn Observed<U2>>)
        where
            T2: 'static,
            U2: 'static,
        {
            fn value(&self, observer: &Rc<dyn Observer>) -> Rc<dyn Value<(T2, U2)>> {
                self.0.value(observer).join(self.1.value(observer))
            }
        }

        Box::new(r)
    }

    fn split(self: Box<Self>) -> Split<T> {
        Split::<T> {
            observed: Rc::from(self),
        }
    }
}

pub trait Value<T> {
    fn get(&self) -> T;
}

struct ValueMap<T, U, F: Fn(T) -> U> {
    underlying: Rc<dyn Value<T>>,
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
    underlying: Rc<dyn Value<T>>,
    other: Rc<dyn Value<U>>,
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
pub trait ValueExt<T> {
    fn map<U, F: Fn(T) -> U>(self: Rc<Self>, adapter: F) -> Rc<dyn Value<U>>
    where
        U: 'static,
        F: 'static;
    fn join<U>(self: Rc<Self>, other: Rc<dyn Value<U>>) -> Rc<dyn Value<(T, U)>>
    where
        U: 'static;
}
impl<T> ValueExt<T> for dyn Value<T>
where
    T: 'static,
{
    fn map<U, F: Fn(T) -> U>(self: Rc<Self>, adapter: F) -> Rc<dyn Value<U>>
    where
        U: 'static,
        F: 'static,
    {
        Rc::new(ValueMap::<T, U, F> {
            underlying: self,
            adapter,
        })
    }

    fn join<U>(self: Rc<Self>, other: Rc<dyn Value<U>>) -> Rc<dyn Value<(T, U)>>
    where
        U: 'static,
    {
        Rc::new(ValueJoin::<T, U> {
            underlying: self,
            other: other,
        })
    }
}

struct StateValueValue<T> {
    observer: Weak<dyn Observer>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> Value<T> for StateValueValue<T> {
    fn get(&self) -> T {
        *self.value.borrow()
    }
}
#[derive(Clone)]
pub struct StateValue<T> {
    observers: Rc<RefCell<Vec<Weak<StateValueValue<T>>>>>,
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

        impl<T2: Copy> Observed<T2> for StateValue<T2>
        where
            T2: 'static,
        {
            fn value(&self, observer: &Rc<dyn Observer>) -> Rc<dyn Value<T2>> {
                let value = Rc::new(StateValueValue {
                    observer: Rc::downgrade(observer),
                    value: self.value.clone(),
                });

                self.observers.borrow_mut().push(Rc::downgrade(&value));

                value
            }
        }

        (setter.clone(), Box::new(setter))
    }

    fn lock_observers(&mut self) -> Vec<Rc<StateValueValue<T>>> {
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

    pub fn set(&mut self, value: T) -> usize {
        *self.value.borrow_mut() = value;
        let observers = self.lock_observers();
        let r = observers.len();
        observers.into_iter().for_each(|x| {
            x.observer.upgrade().map(|y| y.update());
        });

        r
    }
}
