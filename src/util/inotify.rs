use std::path::PathBuf;
use std::sync::mpsc::Sender;
use inotify::{Inotify, WatchMask, EventMask};
use anyhow::Result;

use crate::Event;

pub fn watch_file_for_changes(tx: Sender<Event>, file_path: PathBuf) -> Result<()> {
    let mut inotify = Inotify::init()?;
    inotify.watches().add(file_path, WatchMask::MODIFY)?;

    let mut buffer = [0; 1024];
    loop {
        let events = inotify.read_events_blocking(&mut buffer)?;
        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                tx.send(Event::FileModified).unwrap();
            }
        }
    }
}

