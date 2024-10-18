use libc::user_regs_struct;
use rhai::{CustomType, Engine, TypeBuilder};

use crate::util::maps::MapEntry;

#[derive(Debug, Clone, Copy, CustomType)]
pub struct GPRegisters {
    pub r15: i64,
    pub r14: i64,
    pub r13: i64,
    pub r12: i64,
    pub rbp: i64,
    pub rbx: i64,
    pub r11: i64,
    pub r10: i64,
    pub r9: i64,
    pub r8: i64,
    pub rax: i64,
    pub rcx: i64,
    pub rdx: i64,
    pub rsi: i64,
    pub rdi: i64,
    pub orig_rax: i64,
    pub rip: i64,
    pub cs: i64,
    pub eflags: i64,
    pub rsp: i64,
    pub ss: i64,
    pub fs_base: i64,
    pub gs_base: i64,
    pub ds: i64,
    pub es: i64,
    pub fs: i64,
    pub gs: i64,
}

impl From<user_regs_struct> for GPRegisters {
    fn from(regs: user_regs_struct) -> Self {
        Self {
            r15: regs.r15 as _,
            r14: regs.r14 as _,
            r13: regs.r13 as _,
            r12: regs.r12 as _,
            rbp: regs.rbp as _,
            rbx: regs.rbx as _,
            r11: regs.r11 as _,
            r10: regs.r10 as _,
            r9: regs.r9 as _,
            r8: regs.r8 as _,
            rax: regs.rax as _,
            rcx: regs.rcx as _,
            rdx: regs.rdx as _,
            rsi: regs.rsi as _,
            rdi: regs.rdi as _,
            orig_rax: regs.orig_rax as _,
            rip: regs.rip as _,
            cs: regs.cs as _,
            eflags: regs.eflags as _,
            rsp: regs.rsp as _,
            ss: regs.ss as _,
            fs_base: regs.fs_base as _,
            gs_base: regs.gs_base as _,
            ds: regs.ds as _,
            es: regs.es as _,
            fs: regs.fs as _,
            gs: regs.gs as _,
        }
    }
}

impl Into<user_regs_struct> for GPRegisters {
    fn into(self) -> user_regs_struct {
        user_regs_struct {
            r15: self.r15 as _,
            r14: self.r14 as _,
            r13: self.r13 as _,
            r12: self.r12 as _,
            rbp: self.rbp as _,
            rbx: self.rbx as _,
            r11: self.r11 as _,
            r10: self.r10 as _,
            r9: self.r9 as _,
            r8: self.r8 as _,
            rax: self.rax as _,
            rcx: self.rcx as _,
            rdx: self.rdx as _,
            rsi: self.rsi as _,
            rdi: self.rdi as _,
            orig_rax: self.orig_rax as _,
            rip: self.rip as _,
            cs: self.cs as _,
            eflags: self.eflags as _,
            rsp: self.rsp as _,
            ss: self.ss as _,
            fs_base: self.fs_base as _,
            gs_base: self.gs_base as _,
            ds: self.ds as _,
            es: self.es as _,
            fs: self.fs as _,
            gs: self.gs as _,
        }
    }
}

impl rhai::FuncArgs for GPRegisters {
    fn parse<ARGS: Extend<rhai::Dynamic>>(self, args: &mut ARGS) {
        let mut map = rhai::Map::new();
        map.insert("r15".into(), (self.r15 as i64).into());
        map.insert("r14".into(), (self.r14 as i64).into());
        map.insert("r13".into(), (self.r13 as i64).into());
        map.insert("r12".into(), (self.r12 as i64).into());
        map.insert("rbp".into(), (self.rbp as i64).into());
        map.insert("rbx".into(), (self.rbx as i64).into());
        map.insert("r11".into(), (self.r11 as i64).into());
        map.insert("r10".into(), (self.r10 as i64).into());
        map.insert("r9".into(), (self.r9 as i64).into());
        map.insert("r8".into(), (self.r8 as i64).into());
        map.insert("rax".into(), (self.rax as i64).into());
        map.insert("rcx".into(), (self.rcx as i64).into());
        map.insert("rdx".into(), (self.rdx as i64).into());
        map.insert("rsi".into(), (self.rsi as i64).into());
        map.insert("rdi".into(), (self.rdi as i64).into());
        map.insert("orig_rax".into(), (self.orig_rax as i64).into());
        map.insert("rip".into(), (self.rip as i64).into());
        map.insert("cs".into(), (self.cs as i64).into());
        map.insert("eflags".into(), (self.eflags as i64).into());
        map.insert("rsp".into(), (self.rsp as i64).into());
        map.insert("ss".into(), (self.ss as i64).into());
        map.insert("fs_base".into(), (self.fs_base as i64).into());
        map.insert("gs_base".into(), (self.gs_base as i64).into());
        map.insert("ds".into(), (self.ds as i64).into());
        map.insert("es".into(), (self.es as i64).into());
        map.insert("fs".into(), (self.fs as i64).into());
        map.insert("gs".into(), (self.gs as i64).into());

        args.extend(Some(map.into()));
    }
}

impl From<rhai::Map> for GPRegisters {
    fn from(value: rhai::Map) -> Self {
        Self {
            r15: value.get("r15").unwrap().as_int().unwrap() as i64,
            r14: value.get("r14").unwrap().as_int().unwrap() as i64,
            r13: value.get("r13").unwrap().as_int().unwrap() as i64,
            r12: value.get("r12").unwrap().as_int().unwrap() as i64,
            rbp: value.get("rbp").unwrap().as_int().unwrap() as i64,
            rbx: value.get("rbx").unwrap().as_int().unwrap() as i64,
            r11: value.get("r11").unwrap().as_int().unwrap() as i64,
            r10: value.get("r10").unwrap().as_int().unwrap() as i64,
            r9: value.get("r9").unwrap().as_int().unwrap() as i64,
            r8: value.get("r8").unwrap().as_int().unwrap() as i64,
            rax: value.get("rax").unwrap().as_int().unwrap() as i64,
            rcx: value.get("rcx").unwrap().as_int().unwrap() as i64,
            rdx: value.get("rdx").unwrap().as_int().unwrap() as i64,
            rsi: value.get("rsi").unwrap().as_int().unwrap() as i64,
            rdi: value.get("rdi").unwrap().as_int().unwrap() as i64,
            orig_rax: value.get("orig_rax").unwrap().as_int().unwrap() as i64,
            rip: value.get("rip").unwrap().as_int().unwrap() as i64,
            cs: value.get("cs").unwrap().as_int().unwrap() as i64,
            eflags: value.get("eflags").unwrap().as_int().unwrap() as i64,
            rsp: value.get("rsp").unwrap().as_int().unwrap() as i64,
            ss: value.get("ss").unwrap().as_int().unwrap() as i64,
            fs_base: value.get("fs_base").unwrap().as_int().unwrap() as i64,
            gs_base: value.get("gs_base").unwrap().as_int().unwrap() as i64,
            ds: value.get("ds").unwrap().as_int().unwrap() as i64,
            es: value.get("es").unwrap().as_int().unwrap() as i64,
            fs: value.get("fs").unwrap().as_int().unwrap() as i64,
            gs: value.get("gs").unwrap().as_int().unwrap() as i64,
        }
    }
}

#[derive(Debug, Clone, CustomType)]
pub struct Task {
    pub pid: i64,
    pub name: String,
}

impl rhai::FuncArgs for Task {
    fn parse<ARGS: Extend<rhai::Dynamic>>(self, args: &mut ARGS) {
        let mut map = rhai::Map::new();
        map.insert("pid".into(), (self.pid as i64).into());
        map.insert("name".into(), self.name.into());

        args.extend(Some(map.into()));
    }
}

impl From<&crate::debugger::task::Task> for Task {
    fn from(task: &crate::debugger::task::Task) -> Self {
        Self {
            pid: task.pid as _,
            name: task.name.to_string(),
        }
    }
}

impl From<&mut crate::debugger::task::Task> for Task {
    fn from(task: &mut crate::debugger::task::Task) -> Self {
        Self {
            pid: task.pid as _,
            name: task.name.to_string(),
        }
    }
}

impl Into<rhai::Dynamic> for MapEntry {
    fn into(self) -> rhai::Dynamic {
        let mut map = rhai::Map::new();
        map.insert("start".into(), (self.start as i64).into());
        map.insert("end".into(), (self.end as i64).into());
        map.insert("offset".into(), (self.offset as i64).into());
        map.insert("flags".into(), self.flags.into());
        map.insert("pathname".into(), self.pathname.into());

        map.into()
    }
}

pub fn register_types(engine: &mut Engine) {
    engine.build_type::<Task>();
    engine.build_type::<GPRegisters>();
    engine.build_type::<MapEntry>();
}
