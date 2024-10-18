use std::{
    fs,
    sync::{Arc, Mutex},
};

use debugger::Debugger;
use util::{maps, watcher};

mod debugger;
mod runtime;
mod util;

fn main() {
    let pid = std::env::args().nth(1).unwrap().parse::<u32>().unwrap();
    let mut debugger = Debugger::new();
    let maps = Arc::new(Mutex::new(maps::parse_from_file(pid as i32).unwrap()));

    debugger.attach(pid).unwrap();
    // debugger.add_breakpoint(0x144799ff0);
    debugger.stop_all().expect("Failed to stop all tasks");

    // let mut hwbp = HardwareBreakpoint::new(0x401146, 0);
    // hwbp.enable(pid).unwrap();

    // debugger
    //     .continue_all()
    //     .expect("Failed to continue all tasks");
    // let debugger = Arc::new(Mutex::new(debugger));
    // let mut script = runtime::compile_script(
    //     fs::read_to_string("script.rhai").unwrap(),
    //     pid,
    //     maps.clone(),
    //     debugger.clone(),
    // )
    // .expect("Failed to compile script");
    // loop {
    //     debugger.lock().unwrap().wait(&script).unwrap();
    // }

    let debugger = Arc::new(Mutex::new(debugger));

    let do_recompile = Arc::new(Mutex::new(false));
    watcher::watch_file("script.rhai", do_recompile.clone());

    let mut script = runtime::compile_script(
        fs::read_to_string("script.rhai").unwrap(),
        pid,
        maps.clone(),
        debugger.clone(),
    )
    .expect("Failed to compile script");

    runtime::run_script(&mut script, debugger.clone()).expect("Failed to run script");

    println!("Debugger initialized");

    loop {
        debugger.lock().unwrap().wait(&script).unwrap();

        if *do_recompile.lock().unwrap() {
            *do_recompile.lock().unwrap() = false;
            match runtime::compile_script(
                fs::read_to_string("script.rhai").unwrap(),
                pid,
                maps.clone(),
                debugger.clone(),
            ) {
                Ok(new_script) => {
                    debugger
                        .lock()
                        .unwrap()
                        .stop_all()
                        .expect("Failed to stop all tasks");
                    script = new_script;
                    runtime::run_script(&mut script, debugger.clone())
                        .expect("Failed to run script");
                }
                Err(e) => {
                    eprintln!("Failed to recompile script: {:?}", e);
                }
            }
        }
    }
}
