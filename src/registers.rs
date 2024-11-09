use libc::user_regs_struct;

pub struct Registers {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub eflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub fs_base: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}

impl From<user_regs_struct> for Registers {
    fn from(regs: user_regs_struct) -> Self {
        Self {
            rax: regs.rax,
            rbx: regs.rbx,
            rcx: regs.rcx,
            rdx: regs.rdx,
            rsi: regs.rsi,
            rdi: regs.rdi,
            rbp: regs.rbp,
            rsp: regs.rsp,
            rip: regs.rip,
            eflags: regs.eflags,
            cs: regs.cs,
            fs: regs.fs,
            gs: regs.gs,
            r15: regs.r15,
            r14: regs.r14,
            r13: regs.r13,
            r12: regs.r12,
            r11: regs.r11,
            r10: regs.r10,
            r9: regs.r9,
            r8: regs.r8,
            orig_rax: regs.orig_rax,
            ds: regs.ds,
            es: regs.es,
            fs_base: regs.fs_base,
            gs_base: regs.gs_base,
            ss: regs.ss,
        }
    }
}

impl Into<user_regs_struct> for Registers {
    fn into(self) -> user_regs_struct {
        user_regs_struct {
            r15: self.r15,
            r14: self.r14,
            r13: self.r13,
            r12: self.r12,
            r11: self.r11,
            r10: self.r10,
            r9: self.r9,
            r8: self.r8,
            rax: self.rax,
            rcx: self.rcx,
            rdx: self.rdx,
            rsi: self.rsi,
            rdi: self.rdi,
            orig_rax: self.orig_rax,
            rip: self.rip,
            cs: self.cs,
            eflags: self.eflags,
            rsp: self.rsp,
            ss: self.ss,
            fs_base: self.fs_base,
            gs_base: self.gs_base,
            ds: self.ds,
            es: self.es,
            fs: self.fs,
            gs: self.gs,
            rbp: self.rbp,
            rbx: self.rbx,
        }
    }
}

use libc::user_fpregs_struct;
pub struct FpRegisters {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub st_space: [f32; 32],
    pub xmm_space: [f32; 64],
    pub padding: [u32; 24],
}

impl From<user_fpregs_struct> for FpRegisters {
    fn from(regs: user_fpregs_struct) -> Self {
        Self {
            cwd: regs.cwd,
            swd: regs.swd,
            ftw: regs.ftw,
            fop: regs.fop,
            rip: regs.rip,
            rdp: regs.rdp,
            mxcsr: regs.mxcsr,
            mxcr_mask: regs.mxcr_mask,
            st_space: unsafe { std::mem::transmute(regs.st_space) },
            xmm_space: unsafe { std::mem::transmute(regs.xmm_space) },
            padding: [0; 24],
        }
    }
}

impl From<FpRegisters> for user_fpregs_struct {
    fn from(regs: FpRegisters) -> Self {
        unsafe { std::mem::transmute(regs) }
    }
}
