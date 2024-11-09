use std::io::Write;

use rhai::{Dynamic, Engine};

use crate::{Context, Event};
use anyhow::Result;

fn dynamic_to_bytes(data: Dynamic) -> Result<Vec<u8>> {
    if data.is_array() {
        let data = data
            .into_array()
            .map_err(|_| anyhow::anyhow!("Invalid array"))?;
        let mut bytes = vec!['[' as u8];
        for el in data {
            bytes.extend(dynamic_to_bytes(el)?);
            bytes.push(b',');
        }
        bytes.push(']' as u8);
        Ok(bytes)
    } else if data.is_string() {
        Ok(data
            .into_string()
            .map_err(|_| anyhow::anyhow!("Invalid string"))?
            .into_bytes())
    } else if data.is_blob() {
        Ok(data
            .into_blob()
            .map_err(|_| anyhow::anyhow!("Invalid blob"))?
            .to_vec())
    } else if data.is_bool() {
        Ok(
            if data
                .as_bool()
                .map_err(|_| anyhow::anyhow!("Invalid boolean"))?
            {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            },
        )
    } else if data.is_int() {
        Ok(data
            .as_int()
            .map_err(|_| anyhow::anyhow!("Invalid integer"))?
            .to_string()
            .into_bytes())
    } else if data.is_float() {
        Ok(data
            .as_float()
            .map_err(|_| anyhow::anyhow!("Invalid float"))?
            .to_string()
            .into_bytes())
    } else {
        Ok(format!("{}", data).into_bytes())
    }
}

pub fn create_directories_for_path(path: &str) -> Result<()> {
    let dir = std::path::Path::new(path).parent().ok_or(anyhow::anyhow!("Invalid path"))?;
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn register_functions(engine: &mut Engine, context: Context) {
    engine.register_fn(
        "write_file",
        move |path: String, data: rhai::Dynamic| match dynamic_to_bytes(data) {
            Ok(bytes) => {
                create_directories_for_path(&path).ok();
                std::fs::write(path, bytes).ok();
            }
            _ => (),
        },
    );

    engine.register_fn(
        "append_file",
        move |path: String, data: rhai::Dynamic| match dynamic_to_bytes(data) {
            Ok(bytes) => {
                create_directories_for_path(&path).ok();
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                    file.write_all(&bytes).ok();
                    file.write_all(b"\n").ok();
                }
            }
            _ => (),
        },
    );

    engine.register_fn(
        "input",
        move |prompt: Dynamic| {
            let prompt = prompt.into_string().unwrap_or(String::default());
            print!("{}", prompt);
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            Dynamic::from(input.trim().to_string())
        },
    );

    engine.register_fn("unsigned_shr", |a: i64, b: i64| {
        let a = unsafe { std::mem::transmute::<i64, u64>(a) };
        let b = unsafe { std::mem::transmute::<i64, u64>(b) };
        unsafe { std::mem::transmute::<u64, i64>(a >> b) }
    });

    engine.register_fn("unsigned_shl", |a: i64, b: i64| {
        let a = unsafe { std::mem::transmute::<i64, u64>(a) };
        let b = unsafe { std::mem::transmute::<i64, u64>(b) };
        unsafe { std::mem::transmute::<u64, i64>(a << b) }
    });

    let ctx = context.clone();
    engine.register_fn("quit", move || {
        ctx.tx.send(Event::Exit).unwrap();
    });
}
