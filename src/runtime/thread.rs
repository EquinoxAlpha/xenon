use crate::thread::Thread;
use rhai::{CustomType, FuncArgs, Map, TypeBuilder};

// Rhai-friendly wrapper around a thread
#[derive(Debug, Clone, CustomType)]
pub struct RhaiThread {
    pub pid: i64,
    pub name: String,
}

impl RhaiThread {
    pub fn new(pid: i64, name: String) -> Self {
        Self { pid, name }
    }
}

impl From<&Thread> for RhaiThread {
    fn from(thread: &Thread) -> Self {
        Self::new(thread.pid as i64, thread.name.clone())
    }
}

impl From<&mut Thread> for RhaiThread {
    fn from(thread: &mut Thread) -> Self {
        Self::new(thread.pid as i64, thread.name.clone())
    }
}

impl FuncArgs for RhaiThread {
    fn parse<ARGS: Extend<rhai::Dynamic>>(self, args: &mut ARGS) {
        let mut map = Map::new();
        map.insert("pid".into(), self.pid.into());
        map.insert("name".into(), self.name.into());
        args.extend(Some(map.into()));
    }
}