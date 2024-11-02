use rhai::{Dynamic, Engine};

use crate::util;

use super::Context;

pub fn integer_to_dynamic<T: Into<u64>>(value: Option<T>) -> Dynamic {
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

    engine.register_fn("read_u8", move |address: i64| -> Dynamic {
        integer_to_dynamic(util::mem::read::<u8>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_u16", move |address: i64| -> Dynamic {
        integer_to_dynamic(util::mem::read::<u16>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_u32", move |address: i64| -> Dynamic {
        integer_to_dynamic(util::mem::read::<u32>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_u64", move |address: i64| -> Dynamic {
        integer_to_dynamic(util::mem::read::<u64>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_f32", move |address: i64| -> Dynamic {
        float_to_dynamic(util::mem::read::<f32>(thread_leader, address as _).ok())
    });

    engine.register_fn("read_f64", move |address: i64| -> Dynamic {
        float_to_dynamic(util::mem::read::<f64>(thread_leader, address as _).ok())
    });

    engine.register_fn("write_u8", move |address: i64, value: i64| {
        util::mem::write(thread_leader, address as _, &(value as u8)).ok();
    });

    engine.register_fn("write_u16", move |address: i64, value: i64| {
        util::mem::write(thread_leader, address as _, &(value as u16)).ok();
    });

    engine.register_fn("write_u32", move |address: i64, value: i64| {
        util::mem::write(thread_leader, address as _, &(value as u32)).ok();
    });

    engine.register_fn("write_u64", move |address: i64, value: i64| {
        util::mem::write(thread_leader, address as _, &(value as u64)).ok();
    });

    engine.register_fn("write_f32", move |address: i64, value: f64| {
        util::mem::write(thread_leader, address as _, &(value as f32)).ok();
    });

    engine.register_fn("write_f64", move |address: i64, value: f64| {
        util::mem::write(thread_leader, address as _, &(value as f64)).ok();
    });

    engine.register_fn("read_bytes", move |address: i64, len: i64| -> Dynamic {
        match util::mem::read_bytes(thread_leader, address as _, len as _) {
            Ok(bytes) => Dynamic::from(bytes),
            Err(e) => Dynamic::UNIT,
        }
    });

    engine.register_fn("write_bytes", move |address: i64, bytes: Vec<u8>| {
        util::mem::write_bytes(thread_leader, address as _, &bytes).ok();
    });

    engine.register_fn("read_string", move |address: i64, len: Dynamic| -> Dynamic {
        let len = len.as_int().unwrap_or(256) as usize;
        let Ok(bytes) = util::mem::read_bytes(thread_leader, address as _, len) else {
            return Dynamic::UNIT;
        };
        let nul = bytes.iter().position(|&b| b == 0).unwrap_or(len);
        Dynamic::from(String::from_utf8_lossy(&bytes[..nul]).to_string())
    });

    engine.register_fn("hexdump", move |address: i64, len: i64| -> Dynamic {
        let Ok(bytes) = util::mem::read_bytes(thread_leader, address as _, len as _) else {
            return Dynamic::UNIT;
        };

        let mut result = String::new();
        for (i, byte) in bytes.iter().enumerate() {
            if i % 16 == 0 {
                if i != 0 {
                    result.push(' ');
                    for j in (i - 16)..i {
                        let ch = bytes[j];
                        if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
                            result.push(ch as char);
                        } else {
                            result.push('.');
                        }
                    }
                    result.push('\n');
                }
                result.push_str(&format!("{:08x}: ", address + i as i64));
            }
            result.push_str(&format!("{:02x} ", byte));
            if (i + 1) % 8 == 0 {
                result.push(' ');
            }
        }
        let remaining = bytes.len() % 16;
        if remaining != 0 {
            for _ in 0..(16 - remaining) {
                result.push_str("   ");
            }
            if remaining <= 8 {
                result.push(' ');
            }
            result.push(' ');
            for i in (bytes.len() - remaining)..bytes.len() {
                let ch = bytes[i];
                if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
                    result.push(ch as char);
                } else {
                    result.push('.');
                }
            }
        }
        Dynamic::from(result)
    });

    engine.register_fn("read_ptr_chain", move |address: i64, chain: Vec<i64>| -> Dynamic {
        let mut ptr = address as usize;
        for offset in chain {
            ptr = util::mem::read::<usize>(thread_leader, ptr + offset as usize).unwrap();
        }
        Dynamic::from(ptr)
    });
}
