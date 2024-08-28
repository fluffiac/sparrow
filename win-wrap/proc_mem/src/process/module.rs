use crate::ProcMemError;

/// contains info needed to find the byte pattern in memory
pub struct Signature {
    pub name: String,
    pub pattern: String,
    /// signature offsets for dereferencing
    pub offsets: Vec<isize>,
    /// added to the result
    pub extra: isize,
    /// get the address relative to the module
    pub relative: bool,
    /// read u32 at found address and add it to the result
    pub rip_relative: bool,
    /// added to the rip result
    pub rip_offset: isize,
}

/// contains info about the module and its content in bytes
#[derive(Debug)]
pub struct Module {
    module_name: String,
    module_path: String,
    process_id: u32,
    process_iswow64: bool,
    module_baseaddr: usize,
    module_basesize: usize,
    module_data: Vec<u8>,
}

impl Module {
    pub fn new(
        mname: String,
        mpath: String,
        pid: u32,
        mbaseaddr: usize,
        mbasesize: usize,
        proc: &crate::Process,
    ) -> Self {
        Module {
            module_name: mname,
            module_path: mpath,
            process_id: pid,
            process_iswow64: proc.iswow64(),
            module_baseaddr: mbaseaddr,
            module_basesize: mbasesize,
            module_data: proc.read_module(mbaseaddr, mbasesize).unwrap_or_default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.module_name
    }
    pub fn path(&self) -> &str {
        &self.module_path
    }
    pub fn pid(&self) -> &u32 {
        &self.process_id
    }
    pub fn base_address(&self) -> usize {
        self.module_baseaddr
    }
    pub fn base_size(&self) -> &usize {
        &self.module_basesize
    }
    pub fn data(&self) -> &Vec<u8> {
        &self.module_data
    }

    pub fn get_raw<T: Copy>(&self, mut o: usize, is_relative: bool) -> Option<T> {
        if !is_relative {
            o -= self.module_baseaddr;
        }
        if o + std::mem::size_of::<T>() >= self.module_data.len() {
            return None;
        }
        let ptr = self.module_data.get(o)?;
        let raw: T = unsafe { std::mem::transmute_copy(ptr) };
        Some(raw)
    }
}
