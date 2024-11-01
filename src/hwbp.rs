use std::sync::atomic::AtomicUsize;

use anyhow::Result;

use crate::{thread::Thread, util};

const WORD_SIZE: usize = std::mem::size_of::<usize>();
pub const fn dr_offset(n: usize) -> usize {
    return std::mem::offset_of!(libc::user, u_debugreg) + n * WORD_SIZE;
}

#[allow(unused)]
fn debug_print_dr7(dr7: usize) {
    println!("DR7: {dr7:064b}");
    println!("  L0:   {:b}", dr7 & 1);
    println!("  G0:   {:b}", (dr7 >> 1) & 1);
    println!("  L1:   {:b}", (dr7 >> 2) & 1);
    println!("  G1:   {:b}", (dr7 >> 3) & 1);
    println!("  L2:   {:b}", (dr7 >> 4) & 1);
    println!("  G2:   {:b}", (dr7 >> 5) & 1);
    println!("  L3:   {:b}", (dr7 >> 6) & 1);
    println!("  G3:   {:b}", (dr7 >> 7) & 1);
    println!("  LE:   {:b}", (dr7 >> 8) & 1);
    println!("  GE:   {:b}", (dr7 >> 9) & 1);
    // bit 10 is reserved
    println!("  RTM:  {:b}", (dr7 >> 11) & 1);
    println!("  IR:   {:b}", (dr7 >> 12) & 1);
    println!("  GD:   {:b}", (dr7 >> 13) & 1);
    println!("  R/W0: {:02b}", (dr7 >> 16) & 0b11);
    println!("  LEN0: {:02b}", (dr7 >> 18) & 0b11);
    println!("  R/W1: {:02b}", (dr7 >> 20) & 0b11);
    println!("  LEN1: {:02b}", (dr7 >> 22) & 0b11);
    println!("  R/W2: {:02b}", (dr7 >> 24) & 0b11);
    println!("  LEN2: {:02b}", (dr7 >> 26) & 0b11);
    println!("  R/W3: {:02b}", (dr7 >> 28) & 0b11);
    println!("  LEN3: {:02b}", (dr7 >> 30) & 0b11);
    // the rest is irrelevant (and nonexistent on 32-bit)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HardwareBreakpointType {
    Access,
    Write,
    Read,
    Execute,
}

impl From<HardwareBreakpointType> for u8 {
    fn from(value: HardwareBreakpointType) -> Self {
        match value {
            HardwareBreakpointType::Access => 0x03,
            HardwareBreakpointType::Write => 0x02,
            HardwareBreakpointType::Read => 0x01,
            HardwareBreakpointType::Execute => 0x00,
        }
    }
}

impl TryFrom<u8> for HardwareBreakpointType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(match value {
            0x03 => HardwareBreakpointType::Access,
            0x02 => HardwareBreakpointType::Write,
            0x01 => HardwareBreakpointType::Read,
            0x00 => HardwareBreakpointType::Execute,
            _ => return Err(anyhow::anyhow!("Invalid hardware breakpoint type")),
        })
    }
}

pub static DR_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct HardwareBreakpoint {
    pub addr: u64,
    pub kind: HardwareBreakpointType,
    pub length: usize,
    pub dr: usize,
}

impl HardwareBreakpoint {
    pub fn new(addr: u64, kind: HardwareBreakpointType, length: usize) -> Result<Self> {
        if length != 1 && length != 2 && length != 4 && length != 8 {
            return Err(anyhow::anyhow!("Invalid length"));
        }
        Ok(Self {
            addr,
            kind,
            length,
            dr: DR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        })
    }
}

impl Thread {
    pub fn enable_breakpoint(&mut self, breakpoint: &HardwareBreakpoint) -> Result<()> {
        if !self.is_traced() {
            return Err(anyhow::anyhow!("Thread is not traced"));
        }

        let mut dr7 = util::ptrace::read_user(self.pid, dr_offset(7))?;
        util::ptrace::write_user(self.pid, dr_offset(breakpoint.dr), breakpoint.addr as usize)?; // set the address to watch

        dr7 |= 1 << (2 * breakpoint.dr); // set local enable bit

        let condition = breakpoint.kind as u8 & 0b11; // 0b00 = execute, 0b01 = write, 0b10 = read, 0b11 = read/write
        dr7 &= !(0b11 << (16 + 4 * breakpoint.dr)); // clear bits 16-17 (condition)
        dr7 |= (condition as usize) << (16 + 4 * breakpoint.dr); // ...then set them to the correct value

        // Set length to breakpoint.length
        let length = match breakpoint.length {
            1 => 0b00,
            2 => 0b01,
            8 => 0b10,
            4 => 0b11,
            _ => unreachable!(),
        };
        dr7 &= !(0b11 << (18 + 4 * breakpoint.dr)); // clear bits 18-19 (length)
        dr7 |= length << (18 + 4 * breakpoint.dr); // ...then set them to the correct value

        util::ptrace::write_user(self.pid, dr_offset(7), dr7)?;
        
        Ok(())
    }

    pub fn disable_breakpoint(&mut self, breakpoint: &HardwareBreakpoint) -> Result<()> {
        let mut dr7 = util::ptrace::read_user(self.pid, dr_offset(7))?;

        dr7 &= !(1 << (2 * breakpoint.dr)); // clear local enable bit

        util::ptrace::write_user(self.pid, dr_offset(7), dr7)?;

        Ok(())
    }

    pub fn get_hit_breakpoints(&mut self) -> Result<Vec<usize>> {
        let dr6 = util::ptrace::read_user(self.pid, dr_offset(6))?;
        let mut hit_breakpoints = Vec::new();
        for i in 0..4 {
            if (dr6 & (1 << i)) != 0 {
                hit_breakpoints.push(i);
            }
        }
        Ok(hit_breakpoints)
    }

    pub fn clear_breakpoint_hit(&mut self, index: usize) -> Result<()> {
        if index >= 4 {
            return Err(anyhow::anyhow!("Invalid breakpoint index: {}", index));
        }
        let mut dr6 = util::ptrace::read_user(self.pid, dr_offset(6))?;
        dr6 &= !(1 << index);
        util::ptrace::write_user(self.pid, dr_offset(6), dr6)?;
        Ok(())
    }
}
