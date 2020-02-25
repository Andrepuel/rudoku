use rudoku::rudoku::{
    ChildObserved, Observed, Observer, Value, ValueExt,
};
use rudoku::text::Decorated;
use rudoku::notify::{Read, WatchFile};
use std::cell::RefCell;
use std::rc::Rc;

struct Printer<T> {
    value: RefCell<Option<Box<dyn Value<T>>>>,
}
impl<T: std::fmt::Display> Printer<T>
where
    T: 'static,
{
    fn new(mut observed: Box<dyn Observed<T>>) -> Rc<Printer<T>> {
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

struct Main {
    watch: Box<dyn Observed<()>>,
    text: Read,
    decorated: Decorated,
}
impl Main {
    fn new(path: String) -> (Main, WatchFile) {
        let (watcher, watch) = WatchFile::new(&path);
        let text = Read::new(path);

        (
            Main {
                watch,
                text,
                decorated: Decorated::new(),
            },
            watcher
        )
    }
}
impl Observed<String> for Main {
    fn value(
        &mut self,
        observer: &Rc<dyn Observer>
    ) -> Box<dyn Value<String>> {
        self.decorated.value(observer,
            self.text.value(observer, self.watch.value(observer)).map(|content| {
                let contents = content
                .split("\n")
                .map(|x| String::from(x))
                .collect();

                (1, 1, contents)
            }),
        )
    }
}

fn main() {
    let (main, mut watch) = Main::new(String::from("README.md"));
    let _output = Printer::new(Box::new(main) as Box<dyn Observed<String>>);
    watch.run();
}
