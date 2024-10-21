use std::fs;

use anyhow::Result;
use breakpoint::Breakpoint;
use hardware::{HardwareBreakpoint, DR_COUNTER};
use libc::{PTRACE_EVENT_CLONE, PTRACE_O_TRACECLONE, WSTOPSIG};
use task::Task;

use crate::{
    runtime::{self, types::GPRegisters, Script},
    util::{ptrace, signal},
};

pub mod breakpoint;
pub mod hardware;
pub mod task;

pub struct Debugger {
    pub tasks: Vec<Task>,
    pub breakpoints: Vec<(Breakpoint, rhai::FnPtr)>,
    pub hardware_breakpoints: Vec<(HardwareBreakpoint, rhai::FnPtr)>,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            tasks: Vec::new(),
            breakpoints: Vec::new(),
            hardware_breakpoints: Vec::new(),
        }
    }

    fn add_task(&mut self, pid: u32) {
        let name = fs::read_to_string(format!("/proc/{pid}/comm")).unwrap();
        let mut task = Task::new(pid, self.tasks.len() as u32, name.trim().to_string());

        // for l in fs::read_to_string(format!("/proc/{pid}/status")).unwrap().lines() {
        //     if l.contains("State") {
        //         println!("{}", l);
        //     }
        // }

        // apply our hwbp's to it
        for (hwbp, _) in &mut self.hardware_breakpoints {
            match hwbp.enable(pid as _) {
                Err(_) => return,
                _ => (),
            }
        }

        match ptrace::set_options(pid, PTRACE_O_TRACECLONE) {
            Err(_) => return,
            _ => (),
        }

        match ptrace::cont(pid) {
            Err(_) => return,
            _ => (),
        }

        task.running = true;

        self.tasks.push(task);
    }

    pub fn attach(&mut self, pid: u32) -> Result<()> {
        let mut tasks = vec![];

        for (i, subtask) in fs::read_dir(format!("/proc/{}/task", pid))?.enumerate() {
            let subtask = subtask?;
            let subtask_pid = subtask
                .file_name()
                .to_string_lossy()
                .parse::<u32>()
                .unwrap();
            let subtask_name =
                fs::read_to_string(format!("/proc/{}/task/{}/comm", pid, subtask_pid))?
                    .trim()
                    .to_string();
            tasks.push(Task::new(subtask_pid, i as u32 + 1, subtask_name));
            ptrace::seize(subtask_pid)?;
            ptrace::interrupt(subtask_pid)?;
            signal::wait(subtask_pid)?;
            ptrace::set_options(subtask_pid, PTRACE_O_TRACECLONE)?;
        }

        for task in &mut tasks {
            task.running = false;
        }

        self.tasks.extend(tasks);
        Ok(())
    }

    pub fn clear_breakpoints(&mut self) {
        self.stop_all().expect("Failed to stop all tasks");
        if self.tasks.is_empty() {
            return;
        }

        for (bp, _) in &mut self.breakpoints {
            bp.disable(self.tasks[0].pid)
                .expect("Failed to disable breakpoint");
        }

        for (hwbp, _) in &mut self.hardware_breakpoints {
            for task in &self.tasks {
                hwbp.disable(task.pid)
                    .expect("Failed to disable hardware breakpoint");
            }
        }

        DR_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);

        self.hardware_breakpoints.clear();
        self.breakpoints.clear();
    }

    pub fn detach(&mut self) -> Result<()> {
        self.clear_breakpoints();
        for task in &self.tasks {
            println!("Detaching from task {} ({})", task.id, task.name);
            ptrace::detach(task.pid)?;
        }
        Ok(())
    }

    pub fn continue_all(&mut self) -> Result<()> {
        for task in &mut self.tasks {
            ptrace::cont(task.pid).ok();
            task.running = true;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<()> {
        for task in &mut self.tasks {
            if !task.running {
                continue;
            }
            ptrace::interrupt(task.pid)?;
            signal::wait(task.pid)?;
            task.running = false;
        }
        Ok(())
    }

    pub fn wait(&mut self, script: &Script) -> Result<()> {
        let mut to_add = vec![];
        'iteration: for task in &mut self.tasks {
            if !task.running {
                continue;
            }
            let Ok(status) = signal::wait_nonblock(task.pid) else {
                task.exited = true;
                continue;
            };
            let Some(status) = status else {
                continue;
            };

            match status {
                signal::Status::Exited(code) => {
                    println!("Task {} exited with code {}", task.name, code);
                    task.exited = true;
                }
                signal::Status::Signaled(signal) => {
                    println!("Task {} received terminating signal {}", task.name, signal);
                    task.exited = true;
                }
                signal::Status::Stopped(status) => {
                    // println!("Task {} stopped by signal {}", task.name, WSTOPSIG(status));
                    task.running = false;

                    if (status >> 16) & 0xffff == PTRACE_EVENT_CLONE {
                        let new_pid = ptrace::get_event_message(task.pid)
                            .expect("Failed to get event message");
                        println!("Task {} (PID {}) spawned new task with PID {}", task.name, task.pid, new_pid);
                        signal::wait(new_pid).unwrap();
                        to_add.push((task.pid, new_pid));
                        break;
                    }

                    let stop_code = WSTOPSIG(status);
                    match stop_code {
                        libc::SIGTRAP => {
                            let mut regs = match ptrace::get_regs(task.pid) {
                                Ok(regs) => regs,
                                Err(_) => {
                                    ptrace::cont(task.pid).ok();
                                    continue;
                                }
                            };

                            let Ok(dr6) = ptrace::peek_user(task.pid, hardware::dr_offset(6))
                            else {
                                println!("Failed to read DR6");
                                ptrace::cont(task.pid).ok();
                                continue;
                            };

                            for i in 0..4 {
                                if dr6 & (1 << i) != 0 {
                                    let Some((_, callback)) =
                                        self.hardware_breakpoints.iter().find(|bp| bp.0.dr == i)
                                    else {
                                        // println!("Unknown HWBP");
                                        ptrace::poke_user(
                                            task.pid,
                                            hardware::dr_offset(6),
                                            dr6 & !0xf,
                                        )
                                        .unwrap();
                                        ptrace::cont(task.pid)?;
                                        task.running = true;
                                        continue 'iteration;
                                    };

                                    callback
                                        .call::<()>(
                                            &script.engine,
                                            &script.ast,
                                            (
                                                GPRegisters::from(regs),
                                                runtime::types::Task::from(&mut *task),
                                            ),
                                        )
                                        .expect("Failed to call breakpoint callback");

                                    ptrace::poke_user(task.pid, hardware::dr_offset(6), dr6 & !0xf)
                                        .unwrap();
                                    ptrace::cont(task.pid)?;
                                    task.running = true;
                                    continue 'iteration;
                                }
                            }

                            for (hwbp, callback) in &mut self.hardware_breakpoints {
                                if hwbp.addr == regs.rip as usize {
                                    callback
                                        .call::<()>(
                                            &script.engine,
                                            &script.ast,
                                            (
                                                GPRegisters::from(regs),
                                                runtime::types::Task::from(&mut *task),
                                            ),
                                        )
                                        .expect("Failed to call breakpoint callback");

                                    ptrace::cont(task.pid)?;
                                    task.running = true;
                                    continue 'iteration;
                                }
                            }

                            for (bp, callback) in &mut self.breakpoints {
                                if bp.addr == regs.rip as usize - 1 {
                                    bp.disable(task.pid)?;
                                    regs.rip -= 1;

                                    ptrace::set_regs(task.pid, &regs)?;

                                    callback
                                        .call::<()>(
                                            &script.engine,
                                            &script.ast,
                                            (
                                                GPRegisters::from(regs),
                                                runtime::types::Task::from(&mut *task),
                                            ),
                                        )
                                        .expect("Failed to call breakpoint callback");

                                    ptrace::single_step(task.pid)?;
                                    signal::wait(task.pid)?;
                                    bp.enable(task.pid)?;
                                    ptrace::cont(task.pid)?;
                                    task.running = true;
                                    continue 'iteration;
                                }
                            }

                            // println!("warn: unknown SIGTRAP at 0x{:x}", regs.rip - 1);
                            ptrace::cont(task.pid)?;
                        }
                        libc::SIGINT => {
                            println!("Task {} received SIGINT", task.name);
                            ptrace::detach(task.pid)?;
                            task.exited = true;
                        }
                        _ => {
                            ptrace::cont(task.pid).expect("Failed to continue task");
                        }
                    }
                }
            }
        }
        for (parent, pid) in to_add {
            self.add_task(pid);
            // no, I don't know how to work with the borrowchecker (or rather can't be bothered with working around it properly)
            // but I do know how to make increasingly convoluted workarounds
            self.tasks.iter_mut().for_each(|x| {
                if x.pid == parent {
                    ptrace::cont(parent).ok();
                    x.running = true;
                }
            });
        }
        self.tasks.retain(|task| !task.exited);
        Ok(())
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        self.detach().ok();
    }
}
