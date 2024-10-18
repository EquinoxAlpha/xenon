// Wrapper around the `process_vm_readv` and `process_vm_writev` syscalls.
use anyhow::Result;

/// Read memory from a process.
pub fn read<T>(pid: u32, address: usize) -> Result<T> {
    let mut buffer = std::mem::MaybeUninit::<T>::uninit();
    let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;
    let buffer_size = std::mem::size_of::<T>();

    let result = unsafe {
        libc::process_vm_readv(
            pid as _,
            &libc::iovec {
                iov_base: buffer_ptr,
                iov_len: buffer_size,
            },
            1,
            &libc::iovec {
                iov_base: address as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            0,
        )
    };

    if result != buffer_size as isize {
        return Err(anyhow::anyhow!("Failed to read memory"));
    }

    Ok(unsafe { buffer.assume_init() })
}

/// Write memory to a process.
pub fn write<T>(pid: u32, address: usize, value: T) -> Result<()> {
    let buffer_ptr = &value as *const T as *const std::ffi::c_void;
    let buffer_size = std::mem::size_of::<T>();

    let result = unsafe {
        libc::process_vm_writev(
            pid as _,
            &libc::iovec {
                iov_base: buffer_ptr as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            &libc::iovec {
                iov_base: address as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            0,
        )
    };

    if result != buffer_size as isize {
        return Err(anyhow::anyhow!("Failed to write memory"));
    }

    Ok(())
}

const MAX_STR_LEN: usize = 1024;

/// Convenience function to read a string from a process.
pub fn read_string(pid: u32, address: usize) -> Result<String> {
    let mut buffer = [0u8; MAX_STR_LEN];
    let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;
    let buffer_size = buffer.len();

    let result = unsafe {
        libc::process_vm_readv(
            pid as _,
            &libc::iovec {
                iov_base: buffer_ptr as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            &libc::iovec {
                iov_base: address as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            0,
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to read memory"));
    }

    let len = result as usize;
    let buffer = &buffer[..len];
    let terminator = buffer.iter().position(|&c| c == 0).unwrap_or(len);
    let string = String::from_utf8_lossy(&buffer[..terminator]).to_string();

    Ok(string)
}

/// Read a sequence of bytes from a process.
pub fn read_bytes(pid: u32, address: usize, length: usize) -> Result<Vec<u8>> {
    let mut buffer = vec![0u8; length];
    let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;
    let buffer_size = buffer.len();

    let result = unsafe {
        libc::process_vm_readv(
            pid as _,
            &libc::iovec {
                iov_base: buffer_ptr as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            &libc::iovec {
                iov_base: address as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            0,
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to read memory"));
    }

    Ok(buffer)
}

/// Write a sequence of bytes to a process.
pub fn write_bytes(pid: i32, address: usize, bytes: &[u8]) -> Result<()> {
    let buffer_ptr = bytes.as_ptr() as *const std::ffi::c_void;
    let buffer_size = bytes.len();

    let result = unsafe {
        libc::process_vm_writev(
            pid,
            &libc::iovec {
                iov_base: buffer_ptr as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            &libc::iovec {
                iov_base: address as *mut std::ffi::c_void,
                iov_len: buffer_size,
            },
            1,
            0,
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to write memory"));
    }

    Ok(())
}
