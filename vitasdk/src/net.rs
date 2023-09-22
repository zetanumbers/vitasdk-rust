use std::ffi::CStr;

use vitasdk_sys::{
    sceHttpInit, sceHttpTerm, sceNetCtlInit, sceNetCtlTerm, sceNetInit, sceNetTerm, SceNetInitParam,
};

use crate::{
    error::{sce_result_unit_from_code, SceResult},
    memblock::{MemBlockMut, MemBlockOptions},
    module::{Module, ModuleId},
};

pub struct GlobalState {
    _module: Module,
    _memory: MemBlockMut,
}

impl GlobalState {
    pub fn new() -> SceResult<Self> {
        const MEMORY_SIZE: usize = 0x400000;
        Self::with_memory_size(MEMORY_SIZE)
    }

    pub fn with_memory_size(size: usize) -> SceResult<Self> {
        let module = Module::load(ModuleId::NET)?;
        static MEMORY_NAME: &CStr = match CStr::from_bytes_with_nul(b"SceNetMemory\0") {
            Ok(v) => v,
            Err(_) => panic!(),
        };
        let memory = MemBlockOptions::from_size(size)
            .with_name(MEMORY_NAME)
            .alloc_mut()?;
        let mut param = SceNetInitParam {
            memory: memory.as_mut_ptr().cast(),
            size: size as i32,
            flags: 0,
        };
        sce_result_unit_from_code(unsafe { sceNetInit(&mut param) })?;
        sce_result_unit_from_code(unsafe { sceNetCtlInit() }).map_err(|e| {
            // TODO: Use drop impl
            let _ = unsafe { sceNetTerm() };
            e
        })?;
        Ok(GlobalState {
            _module: module,
            _memory: memory,
        })
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {
        unsafe { sceNetCtlTerm() };
        let _ = unsafe { sceNetTerm() };
    }
}
