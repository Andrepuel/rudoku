use crate::rudoku::{Observed, ObservedExt, StateValue};
use notify::{raw_watcher, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};

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
                value,
            },
            observed,
        )
    }

    pub fn run(&mut self) {
        loop {
            self.receiver.recv().unwrap();
            self.value.set(());
        }
    }
}

pub fn read(path: String, watcher: Box<dyn Observed<()>>) -> Box<dyn Observed<String>> {
    watcher.map(move |_| std::fs::read_to_string(&path).unwrap())
}
