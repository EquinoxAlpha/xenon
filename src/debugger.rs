use crate::hwbp::HardwareBreakpoint;
use crate::thread::{Thread, ThreadState};
use crate::util;
use crate::util::signal::WaitStatus;
use anyhow::Result;
use log::debug;
pub struct Debugger {
    threads: Vec<Thread>,
    breakpoints: Vec<HardwareBreakpoint>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            breakpoints: Vec::new(),
        }
    }

    pub fn attach(&mut self, pid: u32) -> Result<()> {
        let tasks = util::procfs::get_tasks(pid)?;
        for task in tasks {
            let mut thread = Thread::new(task)?;
            thread.attach()?;
            self.threads.push(thread);
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<()> {
        for thread in &mut self.threads {
            if thread.state == ThreadState::Running {
                thread.interrupt()?;
            }
        }
        Ok(())
    }

    pub fn continue_all(&mut self) -> Result<()> {
        for thread in &mut self.threads {
            if thread.is_traced() {
                thread.cont(None)?;
            }
        }
        Ok(())
    }

    // Main loop
    pub fn run(&mut self) -> Result<()> {
        for thread in &mut self.threads {
            let Ok(status) = thread.wait_nonblocking() else {
                thread.detach().ok();
                continue;
            };

            match status {
                WaitStatus::Stopped(info) => {
                    debug!("Thread {} stopped with {}", thread.pid, info);

                    thread.cont(Some(info))?;
                }
                WaitStatus::Exited(signal) => {
                    debug!("Thread {} exited with {}", thread.pid, signal);
                    thread.detach()?;
                }
                WaitStatus::Signaled(signal) => {
                    debug!("Thread {} signaled with {}", thread.pid, signal);
                    thread.detach()?;
                }
                _ => {}
            }
        }
        // Clean up any threads that exited or were detached
        self.threads.retain(|thread| thread.state != ThreadState::Detached && thread.state != ThreadState::Exited);
        Ok(())
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        for thread in &mut self.threads {
            // thread.interrupt().expect("Failed to interrupt thread during debugger shutdown");
            if thread.interrupt().is_ok() {
                for breakpoint in &self.breakpoints {
                    thread
                        .disable_breakpoint(breakpoint)
                        .expect("Failed to disable breakpoint during debugger shutdown");
                }
            }
            thread
                .detach()
                .expect("Failed to detach thread during debugger shutdown");
        }
    }
}
