use proc_mem::{ProcMemError, ProcMemError::*, Process};

use self::MemError::*;
use crate::util::{as_static, as_static_mut};

#[derive(Debug)]
pub enum MemError {
    OpenError,
    ReadFailure,
    Meta(ProcMemError),
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

pub type Result<T> = std::result::Result<T, MemError>;

pub struct Mem {
    proc: Process,
    pub base: usize,
}

impl Mem {
    pub fn init() -> Result<Self> {
        let proc = Process::with_name("hyperdemon.exe").map_err(MemError::from)?;
        let module = proc.module("hyperdemon.exe").map_err(MemError::from)?;
        let base = module.base_address();

        Ok(Self { proc, base })
    }

    pub fn read_val_blocking<T: Default>(&self, addr: usize) -> Result<T> {
        self.proc.read_mem(addr).map_err(MemError::from)
    }

    pub fn read_into_vec_blocking<T: Copy>(
        &self,
        addr: usize,
        vec: &mut Vec<T>,
        len: usize,
    ) -> Result<()> {
        let success = self.proc.read_ptr(vec.as_mut_ptr(), addr, len);

        if !success {
            return Err(MemError::ReadFailure);
        }

        unsafe { vec.set_len(len) }

        Ok(())
    }

    pub fn read_addr_blocking(&self, addr: usize) -> Result<usize> {
        if self.proc.iswow64 {
            Ok(self.read_val_blocking::<u32>(addr)? as usize)
        } else {
            Ok(self.read_val_blocking::<u64>(addr)? as usize)
        }
    }

    pub fn offsets_blocking<const C: usize>(&self, offsets: [usize; C]) -> Result<Option<usize>> {
        match C {
            0 | 1 => panic!("not enough offsets"),
            2 => self
                .read_addr_blocking(offsets[0] + offsets[1])
                .map(|a| (a != 0).then_some(a)),
            _ => {
                let mut addr = self.read_addr_blocking(offsets[0] + offsets[1])?;
                for &offset in &offsets[2..C] {
                    addr = self.read_addr_blocking(addr + offset)?;
                    if addr == 0 {
                        return Ok(None);
                    }
                }
                Ok(Some(addr))
            }
        }
    }

    pub async fn read_val<T: Default + Send + 'static>(&self, addr: usize) -> Result<T> {
        let this = unsafe { as_static(self) };

        tokio::task::spawn_blocking(move || this.read_val_blocking(addr))
            .await
            .unwrap()
    }

    pub async fn read_into_vec<T: Copy + Send + 'static>(
        &self,
        addr: usize,
        vec: &mut Vec<T>,
        len: usize,
    ) -> Result<()> {
        let this = unsafe { as_static(self) };
        let vec = unsafe { as_static_mut(vec) };

        tokio::task::spawn_blocking(move || this.read_into_vec_blocking(addr, vec, len))
            .await
            .unwrap()
    }

    pub async fn read_addr(&self, addr: usize) -> Result<usize> {
        let this = unsafe { as_static(self) };

        tokio::task::spawn_blocking(move || this.read_addr_blocking(addr))
            .await
            .unwrap()
    }

    pub async fn offsets<const C: usize>(&self, offsets: [usize; C]) -> Result<Option<usize>> {
        let this = unsafe { as_static(self) };

        tokio::task::spawn_blocking(move || this.offsets_blocking(offsets))
            .await
            .unwrap()
    }
}
