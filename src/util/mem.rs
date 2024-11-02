use std::mem::{self, MaybeUninit};

use anyhow::Result;
use libc::{process_vm_readv, process_vm_writev, iovec};

pub fn read<T: Sized>(pid: u32, addr: usize) -> Result<T> {
    let mut buf = MaybeUninit::<T>::uninit();
    let local = vec![iovec {
        iov_base: buf.as_mut_ptr() as _,
        iov_len: mem::size_of::<T>(),
    }];
    let remote = vec![iovec {
        iov_base: addr as _,
        iov_len: mem::size_of::<T>(),
    }];
    unsafe {
        if process_vm_readv(pid as _, local.as_ptr(), 1, remote.as_ptr(), 1, 0) == -1 {
            return Err(anyhow::anyhow!("process_vm_readv failed"));
        }
    }
    Ok(unsafe { buf.assume_init() })
}

pub fn write<T: Sized>(pid: u32, addr: usize, value: &T) -> Result<()> {
    let mut buf = MaybeUninit::<T>::uninit();
    let local = vec![iovec {
        iov_base: buf.as_mut_ptr() as _,
        iov_len: mem::size_of::<T>(),
    }];
    let remote = vec![iovec {
        iov_base: addr as _,
        iov_len: mem::size_of::<T>(),
    }];
    unsafe {
        if process_vm_writev(pid as _, local.as_ptr(), 1, remote.as_ptr(), 1, 0) == -1 {
            return Err(anyhow::anyhow!("process_vm_writev failed"));
        }
    }
    Ok(())
}

pub fn read_bytes(pid: u32, addr: usize, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    let local = vec![iovec {
        iov_base: buf.as_mut_ptr() as _,
        iov_len: len,
    }];
    let remote = vec![iovec {
        iov_base: addr as _,
        iov_len: len,
    }];
    unsafe {
        if process_vm_readv(pid as _, local.as_ptr(), 1, remote.as_ptr(), 1, 0) == -1 {
            return Err(anyhow::anyhow!("process_vm_readv failed"));
        }
    }
    Ok(buf)
}

pub fn write_bytes(pid: u32, addr: usize, bytes: &[u8]) -> Result<()> {
    let local = vec![iovec {
        iov_base: bytes.as_ptr() as _,
        iov_len: bytes.len(),
    }];
    let remote = vec![iovec {
        iov_base: addr as _,
        iov_len: bytes.len(),
    }];
    unsafe {
        if process_vm_writev(pid as _, local.as_ptr(), 1, remote.as_ptr(), 1, 0) == -1 {
            return Err(anyhow::anyhow!("process_vm_writev failed"));
        }
    }
    Ok(())
}
