pub mod procfs;
pub mod ptrace;
pub mod signal;
pub mod syscall;
pub mod inotify;
pub mod mem;

pub mod dbg {
    use anyhow::Result;
    use log::debug;
    
    pub fn print_process_state(pid: u32) -> Result<()> {
        let status = std::fs::read_to_string(format!("/proc/{}/status", pid))?;
        for line in status.lines() {
            if line.starts_with("State:") {
                let state = line.split_once(":").unwrap().1;
                debug!("State({}): {}", pid, state.trim());
            }
        }
        Ok(())
    }

    pub fn pretty_print_dr7(dr7: usize) {
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
}
