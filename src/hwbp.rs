use std::sync::atomic::AtomicUsize;

use anyhow::Result;
use log::{debug, error};

use crate::{thread::Thread, util};

const WORD_SIZE: usize = std::mem::size_of::<usize>();
pub const fn dr_offset(n: usize) -> usize {
    return std::mem::offset_of!(libc::user, u_debugreg) + n * WORD_SIZE;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HardwareBreakpointType {
    Execute,
    Read,
    Write,
    Access,
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
    pub address: u64,
    pub kind: HardwareBreakpointType,
    pub length: usize,
    pub dr: usize,
}

impl HardwareBreakpoint {
    pub fn new(address: u64, kind: HardwareBreakpointType, length: usize) -> Result<Self> {
        if length != 1 && length != 2 && length != 4 && length != 8 {
            return Err(anyhow::anyhow!("Invalid length"));
        }
        Ok(Self {
            address,
            kind,
            length,
            dr: DR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 4,
        })
    }
}

impl Thread {
    pub fn set_breakpoint(&mut self, breakpoint: &HardwareBreakpoint) -> Result<()> {
        if !self.is_traced() {
            return Err(anyhow::anyhow!("Thread is not traced"));
        }

        let mut dr7 = util::ptrace::read_user(self.pid, dr_offset(7))?;
        util::ptrace::write_user(self.pid, dr_offset(breakpoint.dr), breakpoint.address as usize)?; // set the address to watch

        dr7 |= 1 << (2 * breakpoint.dr); // set local enable bit

        let condition: u8 = breakpoint.kind.into();
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

    pub fn clear_breakpoint(&mut self, breakpoint: &HardwareBreakpoint) -> Result<()> {
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
