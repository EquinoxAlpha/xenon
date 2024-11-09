use rhai::{CustomType, FuncArgs, Map, TypeBuilder};

use crate::{
    registers::{FpRegisters, Registers},
    util,
};

use super::{Context, RhaiThread};

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

impl From<&RhaiRegisters> for Registers {
    fn from(regs: &RhaiRegisters) -> Self {
        Registers {
            r15: regs.r15 as u64,
            r14: regs.r14 as u64,
            r13: regs.r13 as u64,
            r12: regs.r12 as u64,
            r11: regs.r11 as u64,
            r10: regs.r10 as u64,
            r9: regs.r9 as u64,
            r8: regs.r8 as u64,
            rax: regs.rax as u64,
            rcx: regs.rcx as u64,
            rdx: regs.rdx as u64,
            rsi: regs.rsi as u64,
            rdi: regs.rdi as u64,
            orig_rax: regs.orig_rax as u64,
            rip: regs.rip as u64,
            cs: regs.cs as u64,
            eflags: regs.eflags as u64,
            rsp: regs.rsp as u64,
            ss: regs.ss as u64,
            fs_base: regs.fs_base as u64,
            gs_base: regs.gs_base as u64,
            ds: regs.ds as u64,
            es: regs.es as u64,
            fs: regs.fs as u64,
            gs: regs.gs as u64,
            rbp: regs.rbp as u64,
            rbx: regs.rbx as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, CustomType)]
pub struct RhaiFpRegisters {
    pub cwd: i64,
    pub swd: i64,
    pub ftw: i64,
    pub fop: i64,
    pub rip: i64,
    pub rdp: i64,
    pub mxcsr: i64,
    pub mxcr_mask: i64,
    pub st_space: [f64; 32],
    pub xmm_space: [f64; 64],
    pub padding: [i64; 24],
}

impl From<&FpRegisters> for RhaiFpRegisters {
    fn from(regs: &FpRegisters) -> Self {
        Self {
            cwd: regs.cwd as i64,
            swd: regs.swd as i64,
            ftw: regs.ftw as i64,
            fop: regs.fop as i64,
            rip: regs.rip as i64,
            rdp: regs.rdp as i64,
            mxcsr: regs.mxcsr as i64,
            mxcr_mask: regs.mxcr_mask as i64,
            st_space: regs.st_space.map(|x| x as f64),
            xmm_space: regs.xmm_space.map(|x| x as f64),
            padding: regs.padding.map(|x| x as i64),
        }
    }
}

impl From<&RhaiFpRegisters> for FpRegisters {
    fn from(regs: &RhaiFpRegisters) -> Self {
        Self {
            cwd: regs.cwd as u16,
            swd: regs.swd as u16,
            ftw: regs.ftw as u16,
            fop: regs.fop as u16,
            rip: regs.rip as u64,
            rdp: regs.rdp as u64,
            mxcsr: regs.mxcsr as u32,
            mxcr_mask: regs.mxcr_mask as u32,
            st_space: regs.st_space.map(|x| x as f32),
            xmm_space: regs.xmm_space.map(|x| x as f32),
            padding: regs.padding.map(|x| x as u32),
        }
    }
}

impl FuncArgs for RhaiFpRegisters {
    fn parse<ARGS: Extend<rhai::Dynamic>>(self, args: &mut ARGS) {
        let mut map = Map::new();
        map.insert("cwd".into(), self.cwd.into());
        map.insert("swd".into(), self.swd.into());
        map.insert("ftw".into(), self.ftw.into());
        map.insert("fop".into(), self.fop.into());
        map.insert("rip".into(), self.rip.into());
        map.insert("rdp".into(), self.rdp.into());
        map.insert("mxcsr".into(), self.mxcsr.into());
        map.insert("mxcr_mask".into(), self.mxcr_mask.into());
        map.insert(
            "st_space".into(),
            self.st_space
                .iter()
                .map(|&x| x.into())
                .collect::<Vec<rhai::Dynamic>>()
                .into(),
        );
        for (i, xmm) in self.xmm_space.iter().enumerate() {
            let reg = i / 4;
            let idx = i % 4;
            map.insert(format!("xmm{reg}_{idx}").into(), (*xmm).into());
        }
        map.insert(
            "padding".into(),
            self.padding
                .iter()
                .map(|&x| x.into())
                .collect::<Vec<i64>>()
                .into(),
        );
        args.extend(Some(map.into()));
    }
}

impl TryFrom<rhai::Map> for RhaiFpRegisters {
    type Error = Box<rhai::EvalAltResult>;

    fn try_from(map: rhai::Map) -> Result<Self, Self::Error> {
        let mut xmm_space = [0.0; 64];
        for (i, xmm) in xmm_space.iter_mut().enumerate() {
            let reg = i / 4;
            let idx = i % 4;
            *xmm = map
                .get(format!("xmm{reg}_{idx}").as_str())
                .ok_or("Missing field: xmm{reg}_{idx}")?
                .clone()
                .try_cast::<f64>()
                .ok_or("Not a float")?;
        }
        Ok(Self {
            cwd: map
                .get("cwd")
                .ok_or("Missing field: cwd")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            swd: map
                .get("swd")
                .ok_or("Missing field: swd")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            ftw: map
                .get("ftw")
                .ok_or("Missing field: ftw")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            fop: map
                .get("fop")
                .ok_or("Missing field: fop")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            rip: map
                .get("rip")
                .ok_or("Missing field: rip")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            rdp: map
                .get("rdp")
                .ok_or("Missing field: rdp")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            mxcsr: map
                .get("mxcsr")
                .ok_or("Missing field: mxcsr")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            mxcr_mask: map
                .get("mxcr_mask")
                .ok_or("Missing field: mxcr_mask")?
                .clone()
                .try_cast::<i64>()
                .ok_or("Not an integer")?,
            st_space: {
                let arr = map
                    .get("st_space")
                    .ok_or("Missing field: st_space")?
                    .clone()
                    .try_cast::<Vec<rhai::Dynamic>>()
                    .ok_or("Not an array")?;
                let mut st_space = [0.0; 32];
                for (i, val) in arr.into_iter().enumerate() {
                    st_space[i] = val.try_cast::<f64>().ok_or("Not a float")?;
                }
                st_space
            },
            xmm_space,
            padding: [0; 24],
        })
    }
}

pub fn register_functions(engine: &mut rhai::Engine, context: Context) {
    let ctx = context.clone();
    let thread_leader = ctx.debugger().threads[0].pid;

    // engine.register_fn("set_fp_registers", move |task: RhaiThread, fp_registers: RhaiFpRegisters| {
    //     util::ptrace::set_fp_regs(task.pid as _, &(<FpRegisters as From<&RhaiFpRegisters>>::from(&fp_registers)).into()).ok();
    // });

    engine.register_fn("get_xmm", move |task: RhaiThread, reg: i64, idx: i64| {
        match util::ptrace::get_fp_regs(task.pid as _) {
            Ok(fp_regs) => {
                let value = unsafe {
                    std::mem::transmute::<u32, f32>(fp_regs.xmm_space[reg as usize * 4 + idx as usize])
                };
                (value as f64).into()
            }
            Err(_) => rhai::Dynamic::UNIT
        }
    });

    engine.register_fn(
        "set_xmm",
        move |task: RhaiThread, reg: i64, idx: i64, val: f64| {
            match util::ptrace::get_fp_regs(task.pid as _) {
                Ok(mut fp_regs) => {
                    fp_regs.xmm_space[reg as usize * 4 + idx as usize] =
                        unsafe { std::mem::transmute(val as f32) };
                    if util::ptrace::set_fp_regs(task.pid as _, &fp_regs).is_err() {
                        rhai::Dynamic::UNIT
                    } else {
                        rhai::Dynamic::UNIT
                    }
                }
                Err(_) => rhai::Dynamic::UNIT
            }
        },
    );

    engine.register_fn(
        "set_registers",
        move |task: RhaiThread, regs: RhaiRegisters| {
            util::ptrace::set_regs(
                task.pid as _,
                &(<Registers as From<&RhaiRegisters>>::from(&regs)).into(),
            )
            .ok();
        },
    );
}
