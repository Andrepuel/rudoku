use crate::rudoku::{Observed, ObservedExt, StateValue};
use notify::{raw_watcher, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};

pub struct WatchFile {
    _watcher: RecommendedWatcher,
    receiver: Receiver<RawEvent>,
    value: StateValue<()>,
}
impl WatchFile {
    pub fn new(path: &str) -> (WatchFile, StateValue<()>) {
        let value = StateValue::new(());
        let (tx, receiver) = channel();
        let mut watcher = raw_watcher(tx).unwrap();
        watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

        (
            WatchFile {
                _watcher: watcher,
                receiver,
                value: value.clone(),
            },
            value
        )
    }

    pub fn run(&mut self) {
        loop {
            self.receiver.recv().unwrap();
            self.value.set(());
        }
    }
}

pub fn read(path: String, watcher: impl Observed<T=()> + Clone) -> impl Observed<T=String> + Clone {
    watcher.map(move |_| std::fs::read_to_string(&path).unwrap())
}
