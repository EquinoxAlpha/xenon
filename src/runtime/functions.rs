use std::sync::{Arc, Mutex};

use crate::{
    debugger::{
        breakpoint::Breakpoint,
        hardware::{HardwareBreakpoint, DR_COUNTER},
        Debugger,
    },
    util::{maps::MapEntry, mem, ptrace},
};
use anyhow::Result;
use rhai::{Dynamic, FnPtr};

use super::types::GPRegisters;

fn integer_to_dynamic<T: Into<i64>>(value: Result<T>) -> rhai::Dynamic {
    match value {
        Ok(value) => value.into().into(),
        Err(_) => rhai::Dynamic::UNIT,
    }
}

pub fn register_functions(
    engine: &mut rhai::Engine,
    pid: u32,
    maps: Arc<Mutex<Vec<MapEntry>>>,
    debugger: Arc<Mutex<Debugger>>,
) {
    // Function for adding a breakpoint
    let debugger_arc = debugger.clone();
    engine.register_fn("breakpoint", move |address: i64, callback: rhai::FnPtr| {
        println!("Adding breakpoint at 0x{:x}", address);

        let mut debugger = debugger_arc.lock().unwrap();
        debugger.stop_all().expect("Failed to stop all tasks");

        let mut breakpoint = Breakpoint::new(address as _);
        breakpoint.enable(pid).expect("Failed to enable breakpoint");
        debugger.breakpoints.push((breakpoint, callback));

        debugger
            .continue_all()
            .expect("Failed to continue all tasks");
    });

    // Function for registering a hardware breakpoint
    let debugger_arc = debugger.clone();
    engine.register_fn("hw_breakpoint", move |address: i64, callback: FnPtr| {
        println!("Adding hardware breakpoint at 0x{:x}", address);
        let mut debugger = debugger_arc.lock().unwrap();

        debugger.stop_all().expect("Failed to stop all tasks");

        let mut hwbp = HardwareBreakpoint::new(
            address as _,
            DR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            0,
            1,
        );
        // hwbp.enable(pid).expect("Failed to enable hardware breakpoint");
        for task in debugger.tasks.iter() {
            hwbp.enable(task.pid as _).expect(
                format!("Failed to enable hardware breakpoint for task {}", task.pid).as_str(),
            );
        }
        debugger.hardware_breakpoints.push((hwbp, callback));

        debugger
            .continue_all()
            .expect("Failed to continue all tasks");
    });

    // Watchpoint registration
    let debugger_arc = debugger.clone();
    engine.register_fn("watchpoint", move |address: i64, length: i64, callback: FnPtr| {
        println!("Adding watchpoint at 0x{:x} with length {}", address, length);
        let mut debugger: std::sync::MutexGuard<'_, Debugger> = debugger_arc.lock().unwrap();

        debugger.stop_all().expect("Failed to stop all tasks");

        let mut hwbp = HardwareBreakpoint::new(address as _, DR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed), 0x11, length as _);
        // hwbp.enable(pid).expect("Failed to enable hardware breakpoint");
        for task in debugger.tasks.iter() {
            hwbp.enable(task.pid as _).expect(format!("Failed to enable hardware breakpoint for task {}", task.pid).as_str());
        }
        debugger.hardware_breakpoints.push((hwbp, callback));

        debugger.continue_all().expect("Failed to continue all tasks");
    });

    // Functions for reading integers of 1, 2, 4, and 8 bytes
    engine.register_fn("read64", move |address: i64| -> rhai::Dynamic {
        integer_to_dynamic(mem::read::<i64>(pid, address as _))
    });

    engine.register_fn("read32", move |address: i64| -> rhai::Dynamic {
        integer_to_dynamic(mem::read::<i32>(pid, address as _))
    });

    engine.register_fn("read16", move |address: i64| -> rhai::Dynamic {
        integer_to_dynamic(mem::read::<i16>(pid, address as _))
    });

    engine.register_fn("read8", move |address: i64| -> rhai::Dynamic {
        integer_to_dynamic(mem::read::<i8>(pid, address as _))
    });

    // Functions for writing integers of 1, 2, 4, and 8 bytes
    engine.register_fn("write64", move |address: i64, value: i64| {
        mem::write::<i64>(pid, address as _, value as _).ok();
    });

    engine.register_fn("write32", move |address: i64, value: i64| {
        mem::write::<i32>(pid, address as _, value as _).ok();
    });

    engine.register_fn("write16", move |address: i64, value: i64| {
        mem::write::<i16>(pid, address as _, value as _).ok();
    });

    engine.register_fn("write8", move |address: i64, value: i64| {
        mem::write::<i8>(pid, address as _, value as _).ok();
    });

    // Function for reading a string
    engine.register_fn("read_string", move |address: i64| -> rhai::Dynamic {
        match mem::read_string(pid, address as _) {
            Ok(value) => value.into(),
            Err(_) => rhai::Dynamic::UNIT,
        }
    });

    // Function for reading an arbitrary number of bytes
    engine.register_fn(
        "read_bytes",
        move |address: i64, length: i64| -> rhai::Dynamic {
            match mem::read_bytes(pid, address as _, length as _) {
                Ok(value) => value.into(),
                Err(_) => rhai::Dynamic::UNIT,
            }
        },
    );

    // Function that returns the map entry for a given address
    engine.register_fn("find_map", move |address: i64| -> rhai::Dynamic {
        let maps = maps.lock().unwrap();
        let entry = maps
            .iter()
            .find(|entry| entry.start <= address as usize && entry.end >= address as usize);

        match entry {
            Some(entry) => (entry.clone()).into(),
            None => rhai::Dynamic::UNIT,
        }
    });

    // Function that sets registers
    engine.register_fn("set_regs", move |pid: i64, regs: GPRegisters| {
        ptrace::set_regs(pid as _, &regs.into()).unwrap();
    });

    // Function for printing a hexdump given an address and length
    engine.register_fn("hexdump", move |address: Dynamic, length: i64| -> Dynamic {
        let address = match address.as_int() {
            Ok(address) => address as usize,
            _ => return Dynamic::UNIT,
        };
        let length = length as usize;
        let data = match mem::read_bytes(pid, address as _, length as _) {
            Ok(data) => data,
            Err(_) => return Dynamic::UNIT,
        };
        let mut offset = 0;
        let mut out = String::new();

        while offset < data.len() {
            out.push_str(&format!("{:016x}  ", address + offset));
            for i in 0..16 {
                if offset + i < data.len() {
                    out.push_str(&format!("{:02x} ", data[offset + i]));
                } else {
                    out.push_str("   ");
                }
                if i == 7 {
                    out.push_str(" ");
                }
            }
            out.push_str(" ");
            for i in 0..16 {
                if offset + i < data.len() {
                    let c = data[offset + i] as char;
                    if c.is_ascii_alphanumeric() {
                        out.push(c);
                    } else {
                        out.push('.');
                    }
                }
            }
            out.push('\n');
            offset += 16;
        }
        out.trim().to_string().into()
    });
}
