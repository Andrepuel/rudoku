use rudoku::notify::{read, WatchFile};
use rudoku::rudoku::{fixed, Observed, ObservedExt, Observer, Value};
use rudoku::text::decorated;
use std::cell::RefCell;
use std::rc::Rc;

struct Printer<T> {
    value: RefCell<Option<Rc<dyn Value<T>>>>,
}
impl<T: std::fmt::Display> Printer<T>
where
    T: 'static,
{
    fn new(observed: Box<dyn Observed<T>>) -> Rc<Printer<T>> {
        let r = Rc::new(Printer::<T> {
            value: RefCell::new(None),
        });
        let observer: Rc<dyn Observer> = r.clone();
        let value = observed.value(&observer);
        *r.value.borrow_mut() = Some(value);

        r.update();
        r
    }
}
impl<T: std::fmt::Display> Observer for Printer<T> {
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
