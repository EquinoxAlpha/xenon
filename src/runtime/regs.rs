use rhai::{CustomType, FuncArgs, Map, TypeBuilder};

use crate::registers::Registers;

// Rhai-friendly wrapper around Registers (which in of itself is a wrapper around libc::user_regs_struct)
#[derive(Debug, Clone, Copy, CustomType)]
pub struct RhaiRegisters {
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

impl From<&Registers> for RhaiRegisters {
    fn from(regs: &Registers) -> Self {
        Self {
            r15: regs.r15 as i64,
            r14: regs.r14 as i64,
            r13: regs.r13 as i64,
            r12: regs.r12 as i64,
            rbp: regs.rbp as i64,
            rbx: regs.rbx as i64,
            r11: regs.r11 as i64,
            r10: regs.r10 as i64,
            r9: regs.r9 as i64,
            r8: regs.r8 as i64,
            rax: regs.rax as i64,
            rcx: regs.rcx as i64,
            rdx: regs.rdx as i64,
            rsi: regs.rsi as i64,
            rdi: regs.rdi as i64,
            rip: regs.rip as i64,
            orig_rax: regs.orig_rax as i64,
            cs: regs.cs as i64,
            eflags: regs.eflags as i64,
            rsp: regs.rsp as i64,
            ss: regs.ss as i64,
            fs_base: regs.fs_base as i64,
            gs_base: regs.gs_base as i64,
            ds: regs.ds as i64,
            es: regs.es as i64,
            fs: regs.fs as i64,
            gs: regs.gs as i64,
        }
    }
}

impl FuncArgs for RhaiRegisters {
    fn parse<ARGS: Extend<rhai::Dynamic>>(self, args: &mut ARGS) {
        let mut map = Map::new();
        map.insert("r15".into(), self.r15.into());
        map.insert("r14".into(), self.r14.into());
        map.insert("r13".into(), self.r13.into());
        map.insert("r12".into(), self.r12.into());
        map.insert("rbp".into(), self.rbp.into());
        map.insert("rbx".into(), self.rbx.into());
        map.insert("r11".into(), self.r11.into());
        map.insert("r10".into(), self.r10.into());
        map.insert("r9".into(), self.r9.into());
        map.insert("r8".into(), self.r8.into());
        map.insert("rax".into(), self.rax.into());
        map.insert("rcx".into(), self.rcx.into());
        map.insert("rdx".into(), self.rdx.into());
        map.insert("rsi".into(), self.rsi.into());
        map.insert("rdi".into(), self.rdi.into());
        map.insert("orig_rax".into(), self.orig_rax.into());
        map.insert("rip".into(), self.rip.into());
        map.insert("cs".into(), self.cs.into());
        map.insert("eflags".into(), self.eflags.into());
        map.insert("rsp".into(), self.rsp.into());
        map.insert("ss".into(), self.ss.into());
        map.insert("fs_base".into(), self.fs_base.into());
        map.insert("gs_base".into(), self.gs_base.into());
        map.insert("ds".into(), self.ds.into());
        map.insert("es".into(), self.es.into());
        map.insert("fs".into(), self.fs.into());
        map.insert("gs".into(), self.gs.into());
        args.extend(Some(map.into()));
    }
}
