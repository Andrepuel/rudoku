use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait Observer {
    fn update(&self);
}
pub trait Observed {
    type T;
    type V : Value<T=Self::T>;

    fn value(&self, observer: &Rc<dyn Observer>) -> Self::V;
}
#[derive(Copy, Clone)]
pub struct FixedValue<T: Copy> {
    value: T,
}
impl<T: Copy> Value for FixedValue<T> {
    type T = T;

    fn get(&self) -> Self::T {
        self.value
    }
}
#[derive(Copy, Clone)]
pub struct FixedObserved<T: Copy> {
    value: FixedValue<T>,
}
impl<T: Copy> Observed for FixedObserved<T>
where
    T: 'static,
{
    type T = T;
    type V = FixedValue<T>;

    fn value(&self, _: &Rc<dyn Observer>) -> FixedValue<T> {
        self.value
    }
}
pub fn fixed<T: Copy>(t: T) -> FixedObserved<T>
where
    T: 'static,
{
    FixedObserved::<T> {
        value: FixedValue::<T> { value: t },
    }
}
pub struct ObservedMap<S, F> {
    underlying: S,
    map: F,
}
impl<S: Clone, F: Clone> Clone for ObservedMap<S, F> {
    fn clone(&self) -> Self {
        Self {
            underlying: self.underlying.clone(),
            map: self.map.clone(),
        }
    }
}
impl<S: Copy, F: Copy> Copy for ObservedMap<S, F> {
}
impl<T, S: Observed<T=T>, U, F: Fn(T) -> U + Clone> Observed for ObservedMap<S, F>
    where U: 'static,
    F: 'static,
    T: 'static,
    S::V: Sized,
{
    type T = U;
    type V = ValueMap<S::V, F>;

    fn value(&self, observer: &Rc<dyn Observer>) -> ValueMap<S::V, F> {
        ValueMap::<S::V, F> {
            underlying: self.underlying.value(observer),
            adapter: self.map.clone(),
        }
    }
}
pub struct ObservedJoin<L, R>
{
    left: L,
    right: R,
}
impl<LV, RV, L: Observed<T=LV>, R: Observed<T=RV>> Observed for ObservedJoin<L, R>
    where LV: 'static,
        RV: 'static
{
    type T=(LV, RV);
    type V=ValueJoin<L::V, R::V>;

    fn value(&self, observer: &Rc<dyn Observer>) -> ValueJoin::<L::V, R::V> {
        ValueJoin::<L::V, R::V> {
            left: self.left.value(observer),
            right: self.right.value(observer),
        }
    }
}

pub trait ObservedExt<T> {
    fn map<U, F: Fn(T) -> U + Clone>(self, f: F) -> ObservedMap<Self, F>
    where
        F: 'static,
        U: 'static,
        Self: Sized;
    fn join<U, X: Observed<T=U>>(self, other: X) -> ObservedJoin<Self, X>
    where
        U: 'static,
        Self: Sized;
}
impl<T, TV: Value<T=T>, S: Observed<T=T, V=TV>> ObservedExt<T> for S
where
    T: 'static,
    Self: Sized,
{
    fn map<U, F: Fn(T) -> U + Clone>(self, f: F) -> ObservedMap<Self, F>
    where
        F: 'static,
        U: 'static
    {
        ObservedMap::<Self, F> {
            underlying: self,
            map: f,
        }
    }

    fn join<U, X: Observed<T=U>>(self, other: X) -> ObservedJoin<Self, X>
    where
        U: 'static,
    {
        ObservedJoin::<Self, X> {
            left: self,
            right: other,
        }
    }
}

pub trait Value {
    type T;

    fn get(&self) -> Self::T;
}

pub struct ValueMap<S, F> {
    underlying: S,
    adapter: F,
}
impl<T, S: Value<T=T>, U, F: Fn(T) -> U> Value for ValueMap<S, F>
where
    T: 'static,
    U: 'static,
{
    type T=U;

    fn get(&self) -> U {
        (self.adapter)(self.underlying.get())
    }
}
pub struct ValueJoin<L, R> {
    left: L,
    right: R,
}
impl<T, U, L: Value<T=T>, R: Value<T=U>> Value for ValueJoin<L, R>
where
    T: 'static,
    U: 'static,
{
    type T = (T, U);

    fn get(&self) -> (T, U) {
        (self.left.get(), self.right.get())
    }
}
pub trait ValueExt<T> {
    fn map<U, F: Fn(T) -> U>(self, adapter: F) -> ValueMap<Self, F>
    where
        U: 'static,
        F: 'static,
        Self: Sized;
    fn join<U, R: Value<T=U>>(self, other: R) -> ValueJoin<Self, R>
    where
        U: 'static,
        Self: Sized;
}
impl<T, S: Value<T=T>> ValueExt<T> for S
where
    T: 'static,
    Self: Sized,
{
    fn map<U, F: Fn(T) -> U>(self, adapter: F) -> ValueMap<Self, F>
    where
        U: 'static,
        F: 'static
    {
        ValueMap::<Self, F> {
            underlying: self,
            adapter,
        }
    }

    fn join<U, R: Value<T=U>>(self, other: R) -> ValueJoin<Self, R>
    where
        U: 'static
    {
        ValueJoin::<Self, R> {
            left: self,
            right: other,
        }
    }
}

pub struct StateValueValue<T> {
    observer: Weak<dyn Observer>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> Value for Rc<StateValueValue<T>> {
    type T = T;

    fn get(&self) -> T {
        *self.value.borrow()
    }
}
#[derive(Clone)]
pub struct StateValue<T> {
    observers: Rc<RefCell<Vec<Weak<StateValueValue<T>>>>>,
    value: Rc<RefCell<T>>,
}
impl<T: Copy> Observed for StateValue<T>
where
    T: 'static,
{
    type T = T;
    type V = Rc<StateValueValue<T>>;

    fn value(&self, observer: &Rc<dyn Observer>) -> Rc<StateValueValue<T>> {
        let value = Rc::new(StateValueValue {
            observer: Rc::downgrade(observer),
            value: self.value.clone(),
        });

        self.observers.borrow_mut().push(Rc::downgrade(&value));

        value
    }
}
impl<T: Copy> StateValue<T>
where
    T: 'static,
{
    pub fn new(initial: T) -> StateValue<T> {
        StateValue {
            observers: Rc::new(RefCell::new(Vec::new())),
            value: Rc::new(RefCell::new(initial)),
        }
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
