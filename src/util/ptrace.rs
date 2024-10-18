/// Wrapper around the ptrace syscall.
use libc::{
    ptrace, PTRACE_ATTACH, PTRACE_DETACH, PTRACE_PEEKDATA, PTRACE_POKEDATA, PTRACE_SEIZE,
    PTRACE_SETOPTIONS,
};
use std::{io, mem::MaybeUninit};

/// Attach to a process with the given PID.
pub fn attach(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(PTRACE_ATTACH, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Seize a process with the given PID, tracing it without stopping it.
pub fn seize(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(PTRACE_SEIZE, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Detach from a process with the given PID.
pub fn detach(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(PTRACE_DETACH, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Set ptrace options for a process with the given PID.
pub fn set_options(pid: u32, options: i32) -> io::Result<()> {
    let res = unsafe {
        ptrace(
            PTRACE_SETOPTIONS,
            pid as libc::pid_t,
            0,
            options as *mut libc::c_void,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Read the word at the given address in the given process.
/// Ignores memory protections.
/// Use a syscall like `process_vm_readv` if you need to efficiently read large amounts of memory.
pub fn read(pid: u32, addr: usize) -> io::Result<usize> {
    let res = unsafe {
        ptrace(
            PTRACE_PEEKDATA,
            pid as libc::pid_t,
            addr as *mut libc::c_void,
            0,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(res as usize)
}

/// Write the given word to the given address in the given process.
/// Ignores memory protections.
/// Use a syscall like `process_vm_writev` if you need to efficiently write large amounts of memory.
pub fn write(pid: u32, addr: usize, word: usize) -> io::Result<()> {
    let res = unsafe {
        ptrace(
            PTRACE_POKEDATA,
            pid as libc::pid_t,
            addr as *mut libc::c_void,
            word as *mut libc::c_void,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Continue the execution of a process with the given PID.
/// Fails if the process is not currently stopped.
pub fn cont(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(libc::PTRACE_CONT, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Single-step the execution of a process with the given PID.
/// Fails if the process is not currently stopped.
pub fn single_step(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(libc::PTRACE_SINGLESTEP, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Get the signal information for the given PID.
pub fn get_siginfo(pid: u32) -> io::Result<libc::siginfo_t> {
    let mut siginfo = MaybeUninit::uninit();
    let res = unsafe {
        ptrace(
            libc::PTRACE_GETSIGINFO,
            pid as libc::pid_t,
            0,
            siginfo.as_mut_ptr(),
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { siginfo.assume_init() })
}

/// Get the event message for the given PID.
pub fn get_event_message(pid: u32) -> io::Result<u32> {
    let mut data = MaybeUninit::uninit();
    let res = unsafe { ptrace(libc::PTRACE_GETEVENTMSG, pid as libc::pid_t, 0, &mut data) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { data.assume_init() })
}

/// Fetch general-purpose registers from the given PID.
/// Fails if the process is not currently stopped.
pub fn get_regs(pid: u32) -> io::Result<libc::user_regs_struct> {
    let mut regs = MaybeUninit::uninit();
    let res = unsafe {
        ptrace(
            libc::PTRACE_GETREGS,
            pid as libc::pid_t,
            0,
            regs.as_mut_ptr(),
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { regs.assume_init() })
}

/// Set general-purpose registers for the given PID.
/// Fails if the process is not currently stopped.
pub fn set_regs(pid: u32, regs: &libc::user_regs_struct) -> io::Result<()> {
    let res = unsafe {
        ptrace(
            libc::PTRACE_SETREGS,
            pid as libc::pid_t,
            0,
            regs as *const libc::user_regs_struct as *mut libc::c_void,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Fetch floating-point registers from the given PID.
/// Fails if the process is not currently stopped.
pub fn get_fpregs(pid: u32) -> io::Result<libc::user_fpregs_struct> {
    let mut fpregs = MaybeUninit::uninit();
    let res = unsafe {
        ptrace(
            libc::PTRACE_GETFPREGS,
            pid as libc::pid_t,
            0,
            fpregs.as_mut_ptr(),
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { fpregs.assume_init() })
}

/// Set floating-point registers for the given PID.
/// Fails if the process is not currently stopped.
pub fn set_fpregs(pid: u32, fpregs: &libc::user_fpregs_struct) -> io::Result<()> {
    let res = unsafe {
        ptrace(
            libc::PTRACE_SETFPREGS,
            pid as libc::pid_t,
            0,
            fpregs as *const libc::user_fpregs_struct as *mut libc::c_void,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Attempt to interrupt the process with the given PID.
/// Will not wait for the process to stop.
/// The target process must be attached to via `ptrace::seize`. Calling this on a process that was attached to with `ptrace::attach` will fail.
pub fn interrupt(pid: u32) -> io::Result<()> {
    let res = unsafe { ptrace(libc::PTRACE_INTERRUPT, pid as libc::pid_t, 0, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Implement PTRACE_POKEUSER
/// Offset must be aligned to the size of a usize
pub fn poke_user(pid: u32, offset: usize, data: usize) -> io::Result<()> {
    let res = unsafe {
        ptrace(
            libc::PTRACE_POKEUSER,
            pid as libc::pid_t,
            offset,
            data as *mut libc::c_void,
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Implement PTRACE_PEEKUSER
/// Offset must be aligned to the size of a usize
pub fn peek_user(pid: u32, offset: usize) -> io::Result<usize> {
    let res = unsafe { ptrace(libc::PTRACE_PEEKUSER, pid as libc::pid_t, offset, 0) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(res as usize)
}
