use std::io;

use crate::util::ptrace;

pub struct Breakpoint {
    pub addr: usize,
    pub enabled: bool,
    original_insn: u8,
}

impl Breakpoint {
    pub fn new(addr: usize) -> Breakpoint {
        Breakpoint {
            addr: addr,
            enabled: false,
            original_insn: 0,
        }
    }
    pub fn enable(&mut self, pid: u32) -> io::Result<()> {
        let data = ptrace::read(pid, self.addr)?;
        self.original_insn = (data & 0xff) as u8;
        ptrace::write(pid, self.addr, (data & !0xff) | 0xcc)?;
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self, pid: u32) -> io::Result<()> {
        let data = ptrace::read(pid, self.addr)?;
        ptrace::write(pid, self.addr, (data & !0xff) | self.original_insn as usize)?;
        self.enabled = false;
        Ok(())
    }
}

// the PID needs to be known for a breakpoint to be disabled. as such, breakpoints can't drop themselves
impl Drop for Breakpoint {
    fn drop(&mut self) {
        if self.enabled {
            panic!("Breakpoint was dropped while still enabled. This should never happen!");
        }
    }
}
