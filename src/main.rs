#![feature(try_trait_v2)]
use std::sync::{mpsc, Arc, Mutex};

use anyhow::Result;
use debugger::Debugger;

mod debugger;
mod registers;
mod thread;
mod util;
mod hwbp;
mod runtime;

use hwbp::{HardwareBreakpoint, HardwareBreakpointType, DR_COUNTER};
use log::{debug, error, info};
use runtime::{Context, Script};
use signal_hook::{iterator::Signals, consts::{SIGINT, SIGTERM}};
use util::{inotify::watch_file_for_changes, procfs::MemoryMap};

pub enum Event {
    Exit,
    FileModified,
}

fn main() -> Result<()> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )?;

    let pid = std::env::args().nth(1).unwrap().parse::<u32>()?;
    let script_path = std::env::args().nth(2).unwrap();

    let (tx, rx): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();

    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT, SIGTERM]).expect("Failed to create signals");
        for _ in signals.forever() {
            tx_clone.send(Event::Exit).unwrap();
        }
    });

    let tx_clone = tx.clone();
    let script_path_clone = script_path.clone();
    std::thread::spawn(move || {
        watch_file_for_changes(tx_clone, script_path_clone.into()).unwrap();
    });

    let mut debugger = Debugger::new();
    // debugger.breakpoints.push(HardwareBreakpoint::new(
    //     0x00401256,
    //     HardwareBreakpointType::Execute,
    //     1,
    // )?);
    debugger.attach(pid)?;
    debugger.stop_all()?;

    let context = Context {
        debugger: Arc::new(Mutex::new(debugger)),
        maps: Arc::new(Mutex::new(MemoryMap::parse_maps(pid)?)),
        tx,
    };

    let mut script = Script::new(&std::fs::read_to_string(&script_path)?, context.clone()).unwrap();

    script.run()?;

    context.debugger().apply_breakpoints()?;
    context.debugger().continue_all()?;

    loop {
        match rx.try_recv() {
            Ok(Event::Exit) => {
                break;
            },
            Ok(Event::FileModified) => {
                {
                    context.debugger().stop_all()?;
                    context.debugger().clear_breakpoints()?;
                    context.debugger().callbacks.clear();
                    DR_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
                }
                info!("Reloading script");
                script = Script::new(&std::fs::read_to_string(&script_path)?, context.clone()).unwrap();
                match script.run() {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Error running script: {}", e);
                    }
                }
                {
                    context.debugger().apply_breakpoints()?;
                    context.debugger().continue_all()?;
                }
            },
            _ => {}
        }

        context.debugger().run(&script)?;
    }
    info!("Gracefully exiting...");
    Ok(())
}
