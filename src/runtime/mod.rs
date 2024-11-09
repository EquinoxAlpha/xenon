use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use anyhow::Result;
use rhai::{Engine, AST};

use crate::{debugger::Debugger, util::procfs::MemoryMap, Event};

pub mod mem;
pub mod bp;
mod io;
mod regs;
mod thread;
mod http;
mod flow;

pub use regs::*;
pub use thread::*;

#[derive(Clone)]
pub struct Context {
    pub debugger: Arc<Mutex<Debugger>>,
    pub maps: Arc<Mutex<Vec<MemoryMap>>>,
    pub tx: mpsc::Sender<Event>,
}

impl Context {
    pub fn new(debugger: Arc<Mutex<Debugger>>, maps: Arc<Mutex<Vec<MemoryMap>>>, tx: mpsc::Sender<Event>) -> Self {
        Self { debugger, maps, tx }
    }

    /// Get a lock on the memory maps
    pub fn maps(&self) -> MutexGuard<Vec<MemoryMap>> {
        self.maps.lock().unwrap()
    }

    /// Get a lock on the debugger
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
    engine.build_type::<RhaiFpRegisters>();
    engine.build_type::<RhaiThread>();
}

pub fn register_functions(engine: &mut Engine, context: Context) {
    bp::register_functions(engine, context.clone());
    mem::register_functions(engine, context.clone());
    io::register_functions(engine, context.clone());
    http::register_functions(engine);
    regs::register_functions(engine, context.clone());
    flow::register_functions(engine, context);
}

pub enum RuntimeCallback {
    Breakpoint(usize, rhai::FnPtr),
    ThreadCreated(rhai::FnPtr),
    ThreadExited(rhai::FnPtr),
}
