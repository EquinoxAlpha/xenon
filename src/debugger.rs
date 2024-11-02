use crate::hwbp::{dr_offset, HardwareBreakpoint};
use crate::runtime::{RhaiRegisters, RhaiThread};
use crate::thread::{Thread, ThreadState, DEFAULT_PTRACE_OPTIONS};
use crate::util;
use crate::util::signal::WaitStatus;
use anyhow::Result;
use libc::{PTRACE_EVENT_CLONE, SIGTRAP, WSTOPSIG};
use log::{debug, error, info};

use crate::runtime::{RuntimeCallback, Script};

pub struct Debugger {
    pub threads: Vec<Thread>,
    pub breakpoints: Vec<HardwareBreakpoint>,
    pub callbacks: Vec<RuntimeCallback>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            threads: Vec::new(),
            breakpoints: Vec::new(),
            callbacks: Vec::new(),
        }
    }

    pub fn attach(&mut self, pid: u32) -> Result<()> {
        let tasks = util::procfs::get_tasks(pid)?;
        for task in tasks {
            let mut thread = Thread::new(task)?;
            thread.attach()?;
            debug!("Seized thread {}", thread.pid);
            thread.interrupt()?;
            thread.set_options(DEFAULT_PTRACE_OPTIONS)?;
            thread.cont(None)?;

            self.threads.push(thread);
        }
        info!(
            "Attached to {} threads (Thread leader: {})",
            self.threads.len(),
            self.threads[0].pid
        );
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

    pub fn apply_breakpoints(&mut self) -> Result<()> {
        for thread in &mut self.threads {
            for breakpoint in &self.breakpoints {
                thread.set_breakpoint(breakpoint)?;
                debug!(
                    "Set breakpoint at {:#x} in thread {}",
                    breakpoint.address, thread.pid
                );
            }
        }
        Ok(())
    }

    pub fn clear_breakpoints(&mut self) -> Result<()> {
        for thread in &mut self.threads {
            for breakpoint in &self.breakpoints {
                thread.clear_breakpoint(breakpoint)?;
                debug!(
                    "Cleared breakpoint at {:#x} in thread {}",
                    breakpoint.address, thread.pid
                );
            }
        }
        self.breakpoints.clear();
        Ok(())
    }

    // Main loop
    pub fn run(&mut self, script: &Script) -> Result<()> {
        let mut new_threads = Vec::new();

        for thread in &mut self.threads {
            let Ok(status) = thread.wait_nonblocking() else {
                thread.detach().ok();
                continue;
            };

            match status {
                WaitStatus::Stopped(status) => {
                    let signal = WSTOPSIG(status);
                    debug!(
                        "Thread {} stopped with signal {} (status 0x{:x})",
                        thread.pid, signal, status
                    );

                    if status >> 8 == (SIGTRAP | PTRACE_EVENT_CLONE << 8) {
                        let pid = util::ptrace::get_event_message(thread.pid)?;

                        let mut new_thread = Thread::new(pid as u32)?;
                        new_thread.state = ThreadState::Tracing; // New threads are always traced
                        new_thread.wait()?;
                        new_thread.set_options(DEFAULT_PTRACE_OPTIONS)?;
                        for breakpoint in &self.breakpoints {
                            new_thread.set_breakpoint(breakpoint)?;
                        }
                        for cb in &self.callbacks {
                            match cb {
                                RuntimeCallback::ThreadCreated(cb) => {
                                    if let Err(e) = cb.call::<()>(
                                        &script.engine,
                                        &script.ast,
                                        (RhaiThread::from(&*thread),),
                                    ) {
                                        error!("Error calling thread created callback: {}", e);
                                    }
                                }
                                _ => {}
                            }
                        }
                        new_thread.cont(None)?;

                        debug!(
                            "Thread {} spawned new thread {} (\"{}\")",
                            thread.pid, pid, new_thread.name
                        );

                        new_threads.push(new_thread);

                        thread.cont(None)?;
                    } else if signal == SIGTRAP {
                        let hit_breakpoints = thread.get_hit_breakpoints()?;
                        let registers = thread.get_regs()?;
                        for index in &hit_breakpoints {
                            let Some(breakpoint) = self.breakpoints.iter().find(|x| x.dr == *index)
                            else {
                                continue;
                            };
                            debug!(
                                "Thread {} hit breakpoint {:#x} ({:?})",
                                thread.pid, breakpoint.address, breakpoint.kind
                            );
                            for cb in &self.callbacks {
                                match cb {
                                    RuntimeCallback::Breakpoint(dr, cb) => {
                                        if index == dr {
                                            if let Err(e) = cb.call::<()>(
                                                &script.engine,
                                                &script.ast,
                                                (
                                                    RhaiRegisters::from(&registers),
                                                    RhaiThread::from(&*thread),
                                                ),
                                            ) {
                                                error!(
                                                    "Error calling breakpoint hit callback: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            thread.clear_breakpoint_hit(*index)?;
                        }

                        if hit_breakpoints.is_empty() {
                            thread.cont(Some(signal))?;
                        } else {
                            thread.cont(None)?;
                        }
                    } else {
                        thread.cont(Some(signal))?;
                        debug!("Continued thread {}", thread.pid);
                    }
                }
                WaitStatus::Exited(signal) => {
                    debug!("Thread {} exited with {}", thread.pid, signal);
                    for cb in &self.callbacks {
                        match cb {
                            RuntimeCallback::ThreadExited(cb) => {
                                if let Err(e) = cb.call::<()>(
                                    &script.engine,
                                    &script.ast,
                                    (RhaiThread::from(&*thread),),
                                ) {
                                    error!("Error calling thread exited callback: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                WaitStatus::Signaled(signal) => {
                    debug!("Thread {} signaled with {}", thread.pid, signal);
                }
                _ => {}
            }
        }
        // Clean up any threads that exited or were detached
        self.threads.retain(|thread| {
            !(thread.state == ThreadState::Detached || thread.state == ThreadState::Exited)
        });
        self.threads.extend(new_threads);
        Ok(())
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        debug!("Dropping debugger");
        for thread in &mut self.threads {
            // thread.interrupt().expect("Failed to interrupt thread during debugger shutdown");
            if thread.interrupt().is_ok() {
                for breakpoint in &self.breakpoints {
                    thread
                        .clear_breakpoint(breakpoint)
                        .expect("Failed to disable breakpoint during debugger shutdown");
                }
            }
            thread
                .detach()
                .expect("Failed to detach thread during debugger shutdown");
        }
    }
}
