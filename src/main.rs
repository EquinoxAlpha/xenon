use std::sync::mpsc;

use anyhow::Result;
use debugger::Debugger;

mod debugger;
mod registers;
mod thread;
mod util;
mod hwbp;

use log::info;
use signal_hook::{iterator::Signals, consts::{SIGINT, SIGTERM}};

pub enum Event {
    Exit
}

fn main() -> Result<()> {
    let (tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();

    std::thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).expect("Failed to create signals");
        for _ in signals.forever() {
            tx.send(Event::Exit).unwrap();
        }
    });

    let mut debugger = Debugger::new();

    loop {
        match rx.try_recv() {
            Ok(Event::Exit) => {
                info!("Gracefully exiting...");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
