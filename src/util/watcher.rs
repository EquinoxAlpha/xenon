// Inotify wrapper to watch for file changes in a separate thread.
use std::{path::PathBuf, sync::{Arc, Mutex}};

use inotify::Inotify;

pub fn watch_file(path: impl Into<PathBuf>, set_on_change: Arc<Mutex<bool>>) {
    let path = path.into();
    std::thread::spawn(move || {
        let mut inotify = Inotify::init().unwrap();
        inotify
            .watches()
            .add(path, inotify::WatchMask::MODIFY)
            .unwrap();

        let mut buffer = [0u8; 4096];
        loop {
            let events = inotify.read_events_blocking(&mut buffer).unwrap();
            for event in events {
                if event.mask.contains(inotify::EventMask::MODIFY) {
                    *set_on_change.lock().unwrap() = true;
                }
            }
        }
    });
}
