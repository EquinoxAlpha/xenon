use std::ops::Try;

use log::info;
use rhai::{Dynamic, Engine};

use crate::util;

use super::{Context, RhaiThread};

pub fn integer_to_dynamic<T: Into<i64>>(value: Option<T>) -> Dynamic {
    match value {
        Some(v) => Dynamic::from(v.into()),
        None => Dynamic::UNIT,
    }
}

pub fn float_to_dynamic<T: Into<f64>>(value: Option<T>) -> Dynamic {
    match value {
        Some(v) => Dynamic::from(v.into()),
        None => Dynamic::UNIT,
    }
}

pub fn register_functions(engine: &mut Engine, context: Context) {
    let ctx = context.clone();
    let thread_leader = ctx.debugger().threads[0].pid;

    engine.register_fn("read_i8", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        integer_to_dynamic(util::mem::read::<i8>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_i16", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        integer_to_dynamic(util::mem::read::<i16>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_i32", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        integer_to_dynamic(util::mem::read::<i32>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_i64", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        integer_to_dynamic(util::mem::read::<i64>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_f32", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        float_to_dynamic(util::mem::read::<f32>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_f64", move |address: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        if address < 0 {
            return Dynamic::UNIT;
        }
        float_to_dynamic(util::mem::read::<f64>(thread_leader, address as _).ok())
    });

    engine.register_fn("write_i8", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_int().unwrap_or(-1) as i8;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("write_i16", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_int().unwrap_or(-1) as i16;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("write_i32", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_int().unwrap_or(-1) as i32;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("write_i64", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_int().unwrap_or(-1) as i64;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("write_f32", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_float().unwrap_or(-1.0) as f32;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("write_f64", move |address: Dynamic, value: Dynamic| {
        let address = address.as_int().unwrap_or(-1);
        let value = value.as_float().unwrap_or(-1.0) as f64;
        if address >= 0 {
            util::mem::write(thread_leader, address as _, &value).ok();
        }
    });

    engine.register_fn("read_bytes", move |address: Dynamic, len: Dynamic| -> Dynamic {
        let address = address.as_int().unwrap_or(-1);
        let len = len.as_int().unwrap_or(256).min(32767 * 1000);
        if address < 0 {
            return Dynamic::UNIT;
        }
        match util::mem::read_bytes(thread_leader, address as _, len as _) {
            Ok(bytes) => Dynamic::from(bytes),
            Err(_) => Dynamic::UNIT,
        }
    });

    engine.register_fn("write_bytes", move |address: Dynamic, bytes: Vec<u8>| {
        let address = address.as_int().unwrap_or(-1);
        if address >= 0 {
            util::mem::write_bytes(thread_leader, address as _, &bytes).ok();
        }
    });

    engine.register_fn(
        "read_string",
        move |address: Dynamic, len: Dynamic| -> Dynamic {
            let address = address.as_int().unwrap_or(-1);
            let len = len.as_int().unwrap_or(256).min(32767) as usize;
            if address < 0 {
                return Dynamic::UNIT;
            }
            let Ok(bytes) = util::mem::read_bytes(thread_leader, address as _, len) else {
                return Dynamic::UNIT;
            };
            let nul = bytes.iter().position(|&b| b == 0).unwrap_or(len);
            Dynamic::from(String::from_utf8_lossy(&bytes[..nul]).to_string())
        },
    );

    engine.register_fn("hexdump", move |address: Dynamic, len: Dynamic| -> Dynamic {
        let address = match address.as_int() {
            Ok(address) => address as usize,
            _ => return Dynamic::UNIT,
        };
        let length = len.as_int().unwrap_or(256) as usize;
        let data = match util::mem::read_bytes(thread_leader, address as _, length as _) {
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
                    if c.is_ascii_graphic() {
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

    engine.register_fn(
        "read_ptr_chain",
        move |address: i64, chain: Vec<i64>| -> Dynamic {
            let mut ptr = address as usize;
            for offset in chain {
                ptr = util::mem::read::<usize>(thread_leader, ptr + offset as usize).unwrap();
            }
            Dynamic::from(ptr)
        },
    );

    engine.register_fn("read_stack", move |task: RhaiThread, offset: i64| -> Dynamic {
        let Ok(regs) = util::ptrace::get_regs(task.pid as _) else {
            return Dynamic::UNIT;
        };
        integer_to_dynamic(util::mem::read::<i64>(thread_leader, regs.rsp as usize - offset as usize).ok())
    });

    engine.register_fn("map_entry", move |address: i64| -> Dynamic {
        let maps = context.maps();
        let Some(map) = maps.iter().find(|map| {
            address >= map.start as i64 && address < map.end as i64
        }) else {
            return Dynamic::UNIT;
        };
        let mut result = rhai::Map::new();
        result.insert("start".into(), (map.start as i64).into());
        result.insert("end".into(), (map.end as i64).into());
        result.insert("length".into(), ((map.end - map.start) as i64).into());
        result.insert("permissions".into(), map.permissions.to_string().into());
        result.insert("offset".into(), (map.offset as i64).into());
        result.insert("device".into(), map.device.to_string().into());
        Dynamic::from(result)
    });
}
