use std::fs;

use anyhow::Result;
use breakpoint::Breakpoint;
use hardware::{HardwareBreakpoint, DR_COUNTER};
use libc::{
    PTRACE_EVENT_CLONE, PTRACE_EVENT_EXEC, PTRACE_EVENT_FORK, PTRACE_EVENT_VFORK,
    PTRACE_O_TRACECLONE, PTRACE_O_TRACEEXEC, PTRACE_O_TRACEFORK, PTRACE_O_TRACEVFORK, WSTOPSIG,
};
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
        let name = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap();
        let mut task = Task::new(pid, self.tasks.len() as u32, name.trim().to_string());
        task.running = true;
        // match ptrace::seize(pid) {
        //     Ok(_) => (),
        //     Err(_) => {
        //         // println!("Failed to seize task {}", pid);
        //         let mut task = Task::new(
        //             pid,
        //             self.tasks.len() as u32,
        //             name.trim().to_string(),
        //         );
        //         task.running = true;
        //         self.tasks.push(task);
        //         return;
        //     }
        // }
        // ptrace::interrupt(pid).unwrap();
        // signal::wait(pid).unwrap();
        // ptrace::set_options(
        //     pid,
        //     PTRACE_O_TRACECLONE | PTRACE_O_TRACEFORK | PTRACE_O_TRACEVFORK | PTRACE_O_TRACEEXEC,
        // )
        // .unwrap();
        self.tasks.push(task);
    }

    // pub fn add_breakpoint(&mut self, addr: usize) {
    //     let mut bp = Breakpoint::new(addr);
    //     self.stop_all().expect("Failed to stop all tasks");
    //     bp.enable(self.tasks[0].pid)
    //         .expect("Failed to enable breakpoint");
    //     self.continue_all().expect("Failed to continue all tasks");
    //     self.breakpoints.push(bp);
    // }

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
            ptrace::set_options(
            subtask_pid,
            PTRACE_O_TRACECLONE | PTRACE_O_TRACEFORK | PTRACE_O_TRACEVFORK | PTRACE_O_TRACEEXEC,
            )?;
        }

        for task in &mut tasks {
            task.running = false;
            // println!("Attached to task {} ({})", task.id, task.name);
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
            // println!("Stopped task {} ({})", task.id, task.name);
            task.running = false;
        }
        Ok(())
    }

    pub fn wait(&mut self, script: &Script) -> Result<()> {
        let mut to_add = vec![];
        for task in &mut self.tasks {
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
                        println!("Task {} spawned new task {}", task.name, new_pid);
                        // self.add_task(new_pid);
                        to_add.push(new_pid); // avoid mutating the `tasks` vector while iterating
                        ptrace::cont(new_pid).ok();
                        ptrace::cont(task.pid)?;
                        task.running = true;
                        continue;
                    }
                    if (status >> 16) & 0xffff == PTRACE_EVENT_FORK {
                        let new_pid = ptrace::get_event_message(task.pid)
                            .expect("Failed to get event message");
                        println!("Task {} forked new task {}", task.name, new_pid);
                        // self.add_task(new_pid);
                        to_add.push(new_pid); // avoid mutating the `tasks` vector while iterating
                        ptrace::cont(new_pid).ok();
                        ptrace::cont(task.pid)?;
                        task.running = true;
                        continue;
                    }
                    if (status >> 16) & 0xffff == PTRACE_EVENT_VFORK {
                        let new_pid = ptrace::get_event_message(task.pid)
                            .expect("Failed to get event message");
                        println!("Task {} vforked new task {}", task.name, new_pid);
                        // self.add_task(new_pid);
                        to_add.push(new_pid); // avoid mutating the `tasks` vector while iterating
                        ptrace::cont(task.pid)?;
                        ptrace::cont(new_pid)?;
                        task.running = true;
                        continue;
                    }
                    if (status >> 16) & 0xffff == PTRACE_EVENT_EXEC {
                        println!("Task {} executed new program", task.name);
                        ptrace::cont(task.pid)?;
                        task.running = true;
                        continue;
                    }

                    let stop_code = WSTOPSIG(status);
                    match stop_code {
                        libc::SIGTRAP => {
                            // signal::kill(main_task, libc::SIGSTOP)?; // make sure all other threads are stopped
                            // signal::wait(main_task)?;
                            // match ptrace::interrupt(main_task) {
                            //     Ok(_) => {
                            //         signal::wait(main_task)?;
                            //     },
                            //     _ => (),
                            // }

                            // self.stop_all();

                            let mut regs = match ptrace::get_regs(task.pid) {
                                Ok(regs) => regs,
                                Err(_) => {
                                    // println!("Failed to get registers");
                                    ptrace::cont(task.pid).ok();
                                    continue;
                                }
                            };
                            let mut has_hit_bp = false;

                            for (hwbp, callback) in &mut self.hardware_breakpoints {
                                if hwbp.addr == regs.rip as usize {
                                    //HWBPs don't need to be disabled

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
                                    has_hit_bp = true;
                                    task.running = true;
                                    break;
                                }
                            }

                            if !has_hit_bp {
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
                                        has_hit_bp = true;
                                        break;
                                    }
                                }
                            }

                            if !has_hit_bp {
                                println!("warn: unknown SIGTRAP at 0x{:x}", regs.rip - 1);
                                ptrace::cont(task.pid)?;
                            }
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
        for pid in to_add {
            self.add_task(pid);
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
