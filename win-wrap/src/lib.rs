use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use proc_mem::{ProcMemError, ProcMemError::*, Process};

use self::MemError::*;

pub type Result<T> = std::result::Result<T, MemError>;

#[derive(Debug)]
pub enum MemError {
    OpenError,
    ReadFailure,
    Meta(ProcMemError),
}

impl std::fmt::Display for MemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oops")
    }
}

impl std::error::Error for MemError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<ProcMemError> for MemError {
    fn from(value: ProcMemError) -> Self {
        match value {
            CreateSnapshotFailure => OpenError,
            IterateSnapshotFailure => OpenError,
            ProcessNotFound => OpenError,
            ModuleNotFound => OpenError,
            GetHandleError => OpenError,
            ReadMemoryError => ReadFailure,
            e => Meta(e),
        }
    }
}

#[derive(Clone)]
pub struct Proc {
    proc: Arc<Process>,
    base: usize,
}

#[derive(Clone)]
pub struct Address {
    proc: Proc,
    pub addr: usize,
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x}", self.addr))
    }
}

#[derive(Clone, Copy)]
pub enum Offset<'a> {
    One(usize),
    Many(&'a [usize]),
}

pub trait AsOffset {
    fn as_offset(&self) -> Offset;
}

impl<const N: usize> AsOffset for [usize; N] {
    fn as_offset(&self) -> Offset {
        Offset::Many(self)
    }
}

impl AsOffset for usize {
    fn as_offset(&self) -> Offset {
        Offset::One(*self)
    }
}

use blocking::unblock;
use std::io;

fn read<T: Default>(proc: Proc, addr: usize) -> io::Result<T> {
    proc.proc
        .read_mem(addr)
        .map_err(MemError::from)
        .map_err(io::Error::other)
}

fn vec_read<T: Copy>(proc: Proc, addr: usize, vec: &mut Vec<T>, len: usize) -> io::Result<()> {
    let success = proc.proc.read_ptr(vec.as_mut_ptr(), addr, len);

    if !success {
        return Err(io::Error::other(MemError::ReadFailure));
    }
    
    unsafe { vec.set_len(len) }

    Ok(())
}

fn ptr_read(proc: Proc, addr: usize) -> io::Result<usize> {
    if proc.proc.iswow64 {
        Ok(read::<u32>(proc, addr)? as usize)
    } else {
        Ok(read::<u64>(proc, addr)? as usize)
    }
}

fn with_name(name: &'static str) -> io::Result<Proc> {
    let proc = Process::with_name(name)
        .map_err(MemError::from)
        .map_err(io::Error::other)?;
    let proc = Arc::new(proc);

    let module = proc
        .module(name)
        .map_err(MemError::from)
        .map_err(io::Error::other)?;
    let base = module.base_address();

    Ok(Proc { proc, base })
}

impl Proc {
    pub async fn with_name(name: &'static str) -> io::Result<Self> {
        unblock(|| with_name(name)).await
    }

    pub fn add(&self, offset: usize) -> Address {
        Address {
            proc: self.clone(),
            addr: self.base + offset,
        }
    }

    pub async fn offset(&self, os: impl AsOffset) -> io::Result<Address> {
        self.do_offset(os.as_offset()).await
    }

    pub async fn startup_offset(&self, os: impl AsOffset) -> Address {
        let os = os.as_offset();

        loop {
            let err = self.do_offset(os).await;

            if let Ok(offset) = err {
                break offset;
            }
        }
    }

    async fn do_offset<'a>(&self, os: Offset<'a>) -> io::Result<Address> {
        let offsets = match os {
            Offset::One(offset) => &[offset],
            Offset::Many(offsets) => offsets,
        };

        let mut addr = self.base;
        for offset in offsets {
            let offset = addr + offset;
            let proc = self.clone();
            addr = unblock(move || ptr_read(proc, offset)).await?;
        }

        Ok(Address {
            proc: self.clone(),
            addr,
        })
    }
}

impl Address {
    pub fn add(&self, offset: usize) -> Address {
        Address {
            proc: self.proc.clone(),
            addr: self.addr + offset,
        }
    }

    pub async fn offset(&self, os: impl AsOffset) -> io::Result<Address> {
        self.do_offset(os.as_offset()).await
    }

    pub async fn startup_offset(&self, os: impl AsOffset) -> Address {
        let os = os.as_offset();

        loop {
            let err = self.do_offset(os).await;

            if let Ok(offset) = err {
                break offset;
            }
        }
    }

    async fn do_offset<'a>(&self, os: Offset<'a>) -> io::Result<Address> {
        let offsets = match os {
            Offset::One(offset) => &[offset],
            Offset::Many(offsets) => offsets,
        };

        let mut addr = self.addr;
        for offset in offsets {
            let offset = offset + addr;
            let proc = self.proc.clone();
            addr = unblock(move || ptr_read(proc, offset)).await?;
        }

        Ok(Address {
            proc: self.proc.clone(),
            addr,
        })
    }

    pub async fn read<T: Default + Send + 'static>(&self) -> io::Result<T> {
        let proc = self.proc.clone();
        let addr = self.addr;

        unblock(move || read(proc, addr)).await
    }

    pub async fn vec_read<T: Copy + Send + 'static>(
        &self,
        vec: &mut TVec<T>,
        len: usize,
    ) -> io::Result<()> {
        let proc = self.proc.clone();
        let addr = self.addr;
        let mut vec_handle = vec.handle();

        unblock(move || {
            let mut vec = vec_handle.inner();
            vec_read(proc, addr, &mut vec, len)
        }).await?;

        Ok(())
    }
}

pub struct TVec<T>(Arc<Mutex<Vec<T>>>);
pub struct THandle<T>(Arc<Mutex<Vec<T>>>);

impl<T> From<Vec<T>> for TVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl<T> TVec<T> {
    fn handle(&mut self) -> THandle<T> {
        THandle(self.0.clone())
    }

    pub fn inner(&mut self) -> impl Deref<Target = Vec<T>> + '_ {
        unsafe { self.0.try_lock().unwrap_unchecked() }
    }
}

impl<T> THandle<T> {
    fn inner(&mut self) -> impl DerefMut<Target = Vec<T>> + '_ {
        unsafe { self.0.try_lock().unwrap_unchecked() }
    }
}
