use rudoku::notify::{read, WatchFile};
use rudoku::rudoku::{fixed, Observed, ObservedExt, Observer, Value};
use rudoku::text::decorated;
use std::cell::RefCell;
use std::rc::Rc;

struct Printer<T, V: Value<T=T>> {
    value: RefCell<Option<V>>,
}
impl<T: std::fmt::Display, V: Value<T=T>> Printer<T, V>
where
    T: 'static,
{
    fn new(observed: impl Observed<T=T, V=V>) -> Rc<Printer<T, V>>
    where V: 'static
    {
        let r = Rc::new(Printer::<T, V> {
            value: RefCell::new(None),
        });
        let observer: Rc<dyn Observer> = r.clone();
        let value = observed.value(&observer);
        *r.value.borrow_mut() = Some(value);

        r.update();
        r
    }
}
impl<T: std::fmt::Display, V: Value<T=T>> Observer for Printer<T, V> {
    fn update(&self) {
        println!("{}", self.value.borrow().as_ref().unwrap().get());
    }
}

fn main() {
    let path = String::from("README.md");

    let (mut watcher, watch) = WatchFile::new(&path);

    let main = decorated(
        fixed((1, 1)),
        read(path, watch).map(|content| content.split("\n").map(|x| String::from(x)).collect()),
    );

    let _output = Printer::new(main);
    watcher.run();
}