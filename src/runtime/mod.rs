use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use rhai::{Engine, AST};

use crate::{debugger::Debugger, util::maps::MapEntry};

pub mod functions;
pub mod types;

pub struct Script {
    pub engine: Engine,
    pub ast: AST,
}

pub fn compile_script(
    code: String,
    pid: u32, /* the process ID of the main thread */
    maps: Arc<Mutex<Vec<MapEntry>>>,
    debugger: Arc<Mutex<Debugger>>,
) -> Result<Script> {
    let mut engine = Engine::new();

    types::register_types(&mut engine);
    functions::register_functions(&mut engine, pid, maps, debugger);

    let ast = engine.compile(&code)?;
    Ok(Script { engine, ast })
}

pub fn run_script(script: &mut Script, debugger: Arc<Mutex<Debugger>>) -> Result<()> {
    debugger.lock().unwrap().clear_breakpoints();
    match script.engine.run_ast(&mut script.ast) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error running script: {:?}", e)),
    }
}
