use std::mem::MaybeUninit;

use anyhow::Result;

pub enum WaitStatus {
    Stopped(i32),
    Exited(i32),
    Signaled(i32),
    Running,
}

pub fn wait(pid: u32) -> Result<WaitStatus> {
    let mut status = MaybeUninit::<libc::c_int>::uninit();
    let res = unsafe { libc::waitpid(pid as _, status.as_mut_ptr(), 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to wait for process"));
    }
    let status = unsafe { status.assume_init() };

    if libc::WIFSTOPPED(status) {
        Ok(WaitStatus::Stopped(status))
    } else if libc::WIFEXITED(status) {
        Ok(WaitStatus::Exited(libc::WEXITSTATUS(status)))
    } else if libc::WIFSIGNALED(status) {
        Ok(WaitStatus::Signaled(libc::WTERMSIG(status)))
    } else {
        Err(anyhow::anyhow!("Unknown wait status"))
    }
}

pub fn wait_nonblock(pid: u32) -> Result<WaitStatus> {
    let mut status = MaybeUninit::<libc::c_int>::uninit();
    let res = unsafe { libc::waitpid(pid as _, status.as_mut_ptr(), libc::WNOHANG) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to wait for process"));
    }
    if res == 0 {
        return Ok(WaitStatus::Running);
    }

    let status = unsafe { status.assume_init() };

    if libc::WIFSTOPPED(status) {
        Ok(WaitStatus::Stopped(status))
    } else if libc::WIFEXITED(status) {
        Ok(WaitStatus::Exited(libc::WEXITSTATUS(status)))
    } else if libc::WIFSIGNALED(status) {
        Ok(WaitStatus::Signaled(libc::WTERMSIG(status)))
    } else {
        Err(anyhow::anyhow!("Unknown wait status"))
    }
}