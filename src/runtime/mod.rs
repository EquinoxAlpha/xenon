use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Result;
use rhai::{Engine, AST};

use crate::{debugger::Debugger, util::procfs::MemoryMap};

pub mod mem;
pub mod bp;
mod regs;
mod thread;

pub use regs::*;
pub use thread::*;

#[derive(Clone)]
pub struct Context {
    pub debugger: Arc<Mutex<Debugger>>,
    pub maps: Arc<Mutex<Vec<MemoryMap>>>,
}

impl Context {
    pub fn new(debugger: Arc<Mutex<Debugger>>, maps: Arc<Mutex<Vec<MemoryMap>>>) -> Self {
        Self { debugger, maps }
    }

    pub fn maps(&self) -> MutexGuard<Vec<MemoryMap>> {
        self.maps.lock().unwrap()
    }

    pub fn debugger(&self) -> MutexGuard<Debugger> {
        self.debugger.lock().unwrap()
    }
}

pub struct Script {
    pub engine: Engine,
    pub ast: AST,
}

impl Script {
    pub fn new(source: &str, context: Context) -> Result<Self> {
        let mut engine = Engine::new();
        register_types(&mut engine);
        register_functions(&mut engine, context);
        let ast = engine.compile(source)?;
        Ok(Script { engine, ast })
    }

    pub fn run(&self) -> Result<()> {
        self.engine.run_ast(&self.ast).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

pub fn register_types(engine: &mut Engine) {
    engine.build_type::<RhaiRegisters>();
    engine.build_type::<RhaiThread>();
}

pub fn register_functions(engine: &mut Engine, context: Context) {
    bp::register_functions(engine, context.clone());
    mem::register_functions(engine, context.clone());
}

pub enum RuntimeCallback {
    Breakpoint(usize, rhai::FnPtr),
    ThreadCreated(rhai::FnPtr),
    ThreadExited(rhai::FnPtr),
}
