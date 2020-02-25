use crate::rudoku::{StateValue, Observed, Observer, ChildObserved, Value, ValueExt};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, RawEvent, raw_watcher};
use std::sync::mpsc::{Receiver, channel};
use std::rc::Rc;

pub struct WatchFile {
    _watcher: RecommendedWatcher,
    receiver: Receiver<RawEvent>,
    value: StateValue<()>,
}
impl WatchFile {
    pub fn new(path: &str) -> (WatchFile, Box<dyn Observed<()>>) {
        let (value, observed) = StateValue::new(());
        let (tx, receiver) = channel();
        let mut watcher = raw_watcher(tx).unwrap();
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

        (
            WatchFile {
                _watcher: watcher,
                receiver,
                value
            },
            observed
        )
    }

    pub fn run(&mut self) {
        loop {
            self.receiver.recv().unwrap();
            self.value.set(());
        }
    }
}

pub struct Read {
    path: String,
}
impl Read {
    pub fn new(path: String) -> Read {
        Read {
            path
        }
    }
}
impl ChildObserved<String, ()> for Read {
    fn value(&mut self, _: &Rc<dyn Observer>, input: Box<dyn Value<()>>)
    -> Box<dyn Value<String>>
    {
        let path = self.path.clone();

        input
        .map(move |_| std::fs::read_to_string(&path).unwrap())
    }
}