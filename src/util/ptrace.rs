use anyhow::Result;

use libc::{ptrace, user_regs_struct, PTRACE_ATTACH, PTRACE_CONT, PTRACE_DETACH, PTRACE_GETEVENTMSG, PTRACE_GETREGS, PTRACE_INTERRUPT, PTRACE_PEEKUSER, PTRACE_POKEUSER, PTRACE_SETOPTIONS, PTRACE_SETREGS};
use std::{ffi::c_void, mem::MaybeUninit, ptr};

/// Start tracing the thread with the given PID without interrupting it.
pub fn seize(pid: u32) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_ATTACH, pid, ptr::null_mut::<c_void>(), 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to attach to thread"));
    }
    Ok(())
}

/// Stop tracing the thread with the given PID.
pub fn detach(pid: u32) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_DETACH, pid, ptr::null_mut::<c_void>(), 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to detach from thread"));
    }
    Ok(())
}

/// Set the options for the thread with the given PID.
pub fn set_options(pid: u32, options: i32) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_SETOPTIONS, pid, ptr::null_mut::<c_void>(), options) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to set options"));
    }
    Ok(())
}

/// Interrupt the thread with the given PID. Must be attached to the thread.
pub fn interrupt(pid: u32) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_INTERRUPT, pid, ptr::null_mut::<c_void>(), 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to interrupt thread"));
    }
    Ok(())
}

/// Continue the thread with the given PID.
pub fn cont(pid: u32, signal: Option<i32>) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_CONT, pid, signal.unwrap_or(0), 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to continue execution"));
    }
    Ok(())
}

/// Fetch the registers of a stopped thread with the given PID.
pub fn get_regs(pid: u32) -> Result<user_regs_struct> {
    let mut regs = MaybeUninit::<user_regs_struct>::uninit();
    let res = unsafe { ptrace(PTRACE_GETREGS, pid, regs.as_mut_ptr() as *mut c_void, 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to fetch registers"));
    }
    unsafe { Ok(regs.assume_init()) }
}

/// Set the registers of a running thread with the given PID.
pub fn set_regs(pid: u32, regs: &user_regs_struct) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_SETREGS, pid, regs as *const user_regs_struct as *mut c_void, 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to set registers"));
    }
    Ok(())
}

/// Read memory from the USER area of a running thread with the given PID.
pub fn read_user(pid: u32, addr: usize) -> Result<usize> {
    let mut val = MaybeUninit::<usize>::uninit();
    let res = unsafe { ptrace(PTRACE_PEEKUSER, pid, addr as *mut c_void, val.as_mut_ptr() as *mut c_void) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to read user memory"));
    }
    unsafe { Ok(val.assume_init()) }
}

/// Write memory to the USER area of a running thread with the given PID.
pub fn write_user(pid: u32, addr: usize, val: usize) -> Result<()> {
    let res = unsafe { ptrace(PTRACE_POKEUSER, pid, addr as *mut c_void, val as *mut c_void) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to write user memory"));
    }
    Ok(())
}

pub fn get_event_message(pid: u32) -> Result<u64> {
    let mut val = MaybeUninit::<u64>::uninit();
    let res = unsafe { ptrace(PTRACE_GETEVENTMSG, pid, val.as_mut_ptr() as *mut c_void, 0) };
    if res == -1 {
        return Err(anyhow::anyhow!("Failed to get event message"));
    }
    unsafe { Ok(val.assume_init()) }
}
