use crate::{registers::Registers, util::{self, signal::WaitStatus}};
use anyhow::Result;
use libc::{PTRACE_O_TRACECLONE, PTRACE_O_TRACEEXEC, PTRACE_O_TRACESYSGOOD};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ThreadState {
    Running,
    Stopped,
    Exited,
    Tracing,
    Detached,
}

pub struct Thread {
    pub pid: u32,
    pub name: String,
    pub state: ThreadState,
}

pub const DEFAULT_PTRACE_OPTIONS: i32 = PTRACE_O_TRACESYSGOOD | PTRACE_O_TRACECLONE;

impl Thread {
    pub fn new(pid: u32) -> Result<Self> {
        let path = format!("/proc/{}/comm", pid);
        let name = std::fs::read_to_string(path)?.trim().to_string();
        Ok(Self { pid, name, state: ThreadState::Detached })
    }

    pub fn attach(&mut self) -> Result<()> {
        util::ptrace::seize(self.pid)?;
        self.state = ThreadState::Running;
        Ok(())
    }

    pub fn set_options(&mut self, options: i32) -> Result<()> {
        if self.state != ThreadState::Tracing {
            return Err(anyhow::anyhow!("Thread is not traced"));
        }
        util::ptrace::set_options(self.pid, options)?;
        Ok(())
    }

    pub fn detach(&mut self) -> Result<()> {
        util::ptrace::detach(self.pid)?;
        self.state = ThreadState::Detached;
        Ok(())
    }

    pub fn wait(&mut self) -> Result<WaitStatus> {
        let status = util::signal::wait(self.pid)?;
        self.state = match status {
            WaitStatus::Stopped(_) => ThreadState::Stopped,
            WaitStatus::Exited(_) => ThreadState::Exited,
            WaitStatus::Signaled(_) => ThreadState::Tracing,
            WaitStatus::Running => ThreadState::Running,
        };
        Ok(status)
    }

    pub fn wait_nonblocking(&mut self) -> Result<WaitStatus> {
        let status = util::signal::wait_nonblock(self.pid)?;
        self.state = match status {
            WaitStatus::Stopped(_) => ThreadState::Stopped,
            WaitStatus::Exited(_) => ThreadState::Exited,
            WaitStatus::Signaled(_) => ThreadState::Stopped,
            WaitStatus::Running => ThreadState::Running,
        };
        Ok(status)
    }

    pub fn interrupt(&mut self) -> Result<()> {
        if self.state != ThreadState::Running {
            return Err(anyhow::anyhow!("Thread is not running"));
        }
        util::ptrace::interrupt(self.pid)?;
        self.wait()?;
        self.state = ThreadState::Tracing;
        Ok(())
    }

    pub fn is_traced(&self) -> bool {
        self.state == ThreadState::Tracing
    }

    pub fn cont(&mut self, signal: Option<i32>) -> Result<()> {
        if self.state != ThreadState::Tracing {
            return Err(anyhow::anyhow!("Thread is not traced"));
        }
        util::ptrace::cont(self.pid, signal)?;
        self.state = ThreadState::Running;
        Ok(())
    }

    pub fn get_regs(&mut self) -> Result<Registers> {
        let regs = util::ptrace::get_regs(self.pid)?;
        Ok(regs.into())
    }

    pub fn set_regs(&mut self, regs: Registers) -> Result<()> {
        util::ptrace::set_regs(self.pid, &regs.into())?;
        Ok(())
    }
}
