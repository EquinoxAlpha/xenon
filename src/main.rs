use std::{
    fs,
    sync::{Arc, Mutex},
};

use debugger::Debugger;
use util::{maps, watcher};

mod debugger;
mod runtime;
mod util;

fn print_usage(argv0: String) {
    eprintln!("Usage: {} <pid> <path to script>", argv0);
}

fn main() {
    let argv0 = std::env::args().nth(0).unwrap();
    let pid = match std::env::args().nth(1) {
        Some(pid) => pid.parse::<u32>().unwrap(),
        None => {
            print_usage(argv0);
            return;
        }
    };
    let script_path = match std::env::args().nth(2) {
        Some(path) => path,
        None => {
            print_usage(argv0);
            return;
        }
    };
    let Ok(script_path) = fs::canonicalize(script_path) else {
        eprintln!("Script file not found");
        return;
    };
    
    let mut debugger = Debugger::new();
    let maps = Arc::new(Mutex::new(maps::parse_from_file(pid as i32).unwrap()));

    debugger.attach(pid).unwrap();
    debugger.stop_all().expect("Failed to stop all tasks");
    
    let debugger = Arc::new(Mutex::new(debugger));

    let do_recompile = Arc::new(Mutex::new(false));
    watcher::watch_file(&script_path, do_recompile.clone());

    let mut script = runtime::compile_script(
        fs::read_to_string(&script_path).unwrap(),
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
                fs::read_to_string(&script_path).unwrap(),
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
