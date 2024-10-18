use std::io;

use libc::{WEXITSTATUS, WIFEXITED, WIFSIGNALED, WIFSTOPPED, WTERMSIG};

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Exited(i32),
    Signaled(i32),
    Stopped(i32),
}

/// Wait for a signal to be received from the given PID.
pub fn wait(pid: u32) -> io::Result<Status> {
    let mut status = 0;
    let res = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    if WIFEXITED(status) {
        return Ok(Status::Exited(WEXITSTATUS(status)));
    }
    if WIFSIGNALED(status) {
        return Ok(Status::Signaled(WTERMSIG(status)));
    }
    if WIFSTOPPED(status) {
        return Ok(Status::Stopped(status));
    }
    return Err(io::Error::new(io::ErrorKind::Other, "unknown wait status"));
}

/// Wait for a signal to be received from the given PID. Will not block.
pub fn wait_nonblock(pid: u32) -> io::Result<Option<Status>> {
    let mut status = 0;
    let res = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    if res == 0 {
        return Ok(None);
    }
    if WIFEXITED(status) {
        return Ok(Some(Status::Exited(WEXITSTATUS(status))));
    }
    if WIFSIGNALED(status) {
        return Ok(Some(Status::Signaled(WTERMSIG(status))));
    }
    if WIFSTOPPED(status) {
        return Ok(Some(Status::Stopped(status)));
    }
    return Err(io::Error::new(io::ErrorKind::Other, "unknown wait status"));
}

/// Signal a process with the given PID.
pub fn kill(pid: u32, signal: i32) -> io::Result<()> {
    let res = unsafe { libc::kill(pid as libc::pid_t, signal) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
