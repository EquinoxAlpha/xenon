// x86 Hardware breakpoint implementation
use std::sync::atomic::AtomicUsize;

use crate::util::ptrace;

use anyhow::Result;

pub static DR_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct HardwareBreakpoint {
    pub addr: usize,
    pub enabled: bool,
    pub length: usize,
    pub condition: usize,
    pub dr: usize,
}

const WORD_SIZE: usize = std::mem::size_of::<usize>();
const fn dr_offset(n: usize) -> usize {
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

impl HardwareBreakpoint {
    pub fn new(addr: usize, dr: usize, condition: usize, length: usize) -> HardwareBreakpoint {
        assert!(dr <= 3, "Invalid debug register");
        assert!(
            length == 1 || length == 2 || length == 4 || length == 8,
            "Invalid length"
        );

        HardwareBreakpoint {
            addr,
            enabled: false,
            // mask: 0,
            condition,
            length,
            dr,
        }
    }

    pub fn enable(&mut self, pid: u32) -> Result<()> {
        self.enabled = true;

        let mut dr7 = ptrace::peek_user(pid, dr_offset(7))?;
        ptrace::poke_user(pid, dr_offset(self.dr), self.addr)?; // set the address to watch

        dr7 |= 1 << (2 * self.dr); // set local enable bit

        let condition = self.condition & 0b11; // 0b00 = execute, 0b01 = write, 0b10 = read, 0b11 = read/write
        dr7 &= !(0b11 << (16 + 4 * self.dr)); // clear bits 16-17 (condition)
        dr7 |= condition << (16 + 4 * self.dr); // ...then set them to the correct value

        // Set length to self.length
        let length = match self.length {
            1 => 0b00,
            2 => 0b01,
            8 => 0b10,
            4 => 0b11,
            _ => unreachable!(),
        };
        dr7 &= !(0b11 << (18 + 4 * self.dr)); // clear bits 18-19 (length)
        dr7 |= length << (18 + 4 * self.dr); // ...then set them to the correct value

        ptrace::poke_user(pid, dr_offset(7), dr7)?;

        Ok(())
    }

    pub fn disable(&mut self, pid: u32) -> Result<()> {
        self.enabled = false;

        let mut dr7 = ptrace::peek_user(pid, dr_offset(7))?;

        dr7 &= !(1 << (2 * self.dr)); // clear local enable bit

        ptrace::poke_user(pid, dr_offset(7), dr7)?;

        Ok(())
    }
}

// the PID needs to be known for a breakpoint to be disabled. as such, breakpoints can't drop themselves
impl Drop for HardwareBreakpoint {
    fn drop(&mut self) {
        if self.enabled {
            panic!("Hardware breakpoint was dropped while still enabled. This should never happen!");
        }
    }
}
