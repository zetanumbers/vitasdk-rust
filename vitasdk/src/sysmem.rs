use core::{alloc::Layout, ffi::CStr, mem, ptr};

use vitasdk_sys::{
    sceKernelAllocMemBlock, sceKernelFreeMemBlock, sceKernelGetMemBlockBase,
    SceKernelAllocMemBlockOpt, SceKernelMemBlockType, SCE_KERNEL_ALLOC_MEMBLOCK_ATTR_HAS_ALIGNMENT,
    SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_KERNEL_MEMBLOCK_TYPE_USER_MAIN_RW,
};

use crate::{
    error::{sce_result_uid_from_code, sce_result_unit_from_code, SceResult},
    types::Uid,
};

#[derive(Debug)]
pub struct MemBlockMut {
    inner: MemBlockUninitMut,
}

impl AsRef<[u8]> for MemBlockMut {
    fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.inner.as_ptr(), self.inner.len()) }
    }
}

impl AsMut<[u8]> for MemBlockMut {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.inner.as_mut_ptr(), self.inner.len()) }
    }
}

impl core::borrow::Borrow<[u8]> for MemBlockMut {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl core::borrow::BorrowMut<[u8]> for MemBlockMut {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl core::ops::Deref for MemBlockMut {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl core::ops::DerefMut for MemBlockMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl MemBlockMut {
    pub unsafe fn from_raw(raw: MemBlockRaw) -> SceResult<Self> {
        Ok(MemBlockMut {
            inner: MemBlockUninitMut::from_raw(raw)?,
        })
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_raw(self) -> MemBlockRaw {
        self.inner.into_raw()
    }

    pub fn into_uninit(self) -> MemBlockUninitMut {
        self.inner
    }

    pub fn as_raw(&self) -> &MemBlockRaw {
        self.inner.as_raw()
    }

    pub fn as_uninit(&self) -> &MemBlockUninitMut {
        &self.inner
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn free(self) -> SceResult<()> {
        self.inner.free()
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.inner.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    pub fn mem_partition(&self) -> MemPartition {
        self.inner.mem_partition()
    }
}

#[derive(Debug)]
pub struct MemBlockUninitMut {
    raw: MemBlockRaw,
    base: *mut u8,
}

impl AsRef<[mem::MaybeUninit<u8>]> for MemBlockUninitMut {
    fn as_ref(&self) -> &[mem::MaybeUninit<u8>] {
        unsafe { core::slice::from_raw_parts(self.base.cast(), self.raw.len) }
    }
}

impl AsMut<[mem::MaybeUninit<u8>]> for MemBlockUninitMut {
    fn as_mut(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        unsafe { core::slice::from_raw_parts_mut(self.base.cast(), self.raw.len) }
    }
}

impl core::borrow::Borrow<[mem::MaybeUninit<u8>]> for MemBlockUninitMut {
    fn borrow(&self) -> &[mem::MaybeUninit<u8>] {
        self.as_ref()
    }
}

impl core::borrow::BorrowMut<[mem::MaybeUninit<u8>]> for MemBlockUninitMut {
    fn borrow_mut(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        self.as_mut()
    }
}

impl core::ops::Deref for MemBlockUninitMut {
    type Target = [mem::MaybeUninit<u8>];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl core::ops::DerefMut for MemBlockUninitMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl MemBlockUninitMut {
    pub unsafe fn from_raw(raw: MemBlockRaw) -> SceResult<Self> {
        Ok(MemBlockUninitMut {
            base: raw.get_base()?,
            raw,
        })
    }

    pub unsafe fn assume_init(self) -> MemBlockMut {
        MemBlockMut { inner: self }
    }

    pub fn fill_init(self, value: u8) -> MemBlockMut {
        unsafe {
            self.as_mut_ptr().write_bytes(value, self.len());
            self.assume_init()
        }
    }

    #[cfg(feature = "dmac")]
    #[cfg_attr(docsrs, doc(cfg(feature = "dmac")))]
    pub fn dmac_fill_init(mut self, value: u8) -> SceResult<MemBlockMut> {
        crate::dmac::DmacSliceFillExt::dmac_fill(self.as_mut(), value)?;
        unsafe { Ok(self.assume_init()) }
    }

    pub fn copy_from_slice_init(mut self, src: &[u8]) -> MemBlockMut {
        let src = bytes_as_maybe_uninit(src);
        self.copy_from_slice(src);
        unsafe { self.assume_init() }
    }

    #[cfg(feature = "dmac")]
    #[cfg_attr(docsrs, doc(cfg(feature = "dmac")))]
    pub fn dmac_copy_from_slice_init(mut self, src: &[u8]) -> SceResult<MemBlockMut> {
        let src = bytes_as_maybe_uninit(src);
        crate::dmac::DmacSliceCopyExt::dmac_copy_from_slice(&mut *self, src)?;
        unsafe { Ok(self.assume_init()) }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn into_raw(self) -> MemBlockRaw {
        self.raw
    }

    pub fn as_raw(&self) -> &MemBlockRaw {
        &self.raw
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn free(self) -> SceResult<()> {
        self.raw.free()
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.base
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.base.cast_const()
    }

    pub fn mem_partition(&self) -> MemPartition {
        self.raw.mem_partition()
    }
}

fn bytes_as_maybe_uninit(slice: &[u8]) -> &[mem::MaybeUninit<u8>] {
    // SAFETY: MaybeUninit
    unsafe {
        core::slice::from_raw_parts(slice.as_ptr().cast::<mem::MaybeUninit<u8>>(), slice.len())
    }
}

#[derive(Debug)]
pub struct MemBlockRaw {
    uid: Uid,
    len: usize,
    mem_partition: MemPartition,
}

impl From<MemBlockMut> for MemBlockRaw {
    fn from(value: MemBlockMut) -> Self {
        value.into_raw()
    }
}

impl MemBlockRaw {
    pub fn get_base(&self) -> SceResult<*mut u8> {
        let mut base = ptr::null_mut();
        sce_result_unit_from_code(unsafe { sceKernelGetMemBlockBase(self.uid.get(), &mut base) })?;
        Ok(base.cast())
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn mem_partition(&self) -> MemPartition {
        self.mem_partition
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn free(self) -> SceResult<()> {
        mem::ManuallyDrop::new(self).free_()
    }

    fn free_(&mut self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { sceKernelFreeMemBlock(self.uid.get()) })
    }
}

impl Drop for MemBlockRaw {
    fn drop(&mut self) {
        let _ = self.free_();
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub struct MemBlockOptions<'a> {
    name: &'a CStr,
    mem_partition: MemPartition,
    size: usize,
    alignment: Option<usize>,
}

impl MemBlockOptions<'_> {
    pub fn from_size(size: usize) -> Self {
        MemBlockOptions {
            name: <_>::default(),
            mem_partition: <_>::default(),
            size,
            alignment: None,
        }
    }

    pub fn from_layout(layout: Layout) -> Self {
        MemBlockOptions {
            name: <_>::default(),
            mem_partition: <_>::default(),
            size: layout.size(),
            alignment: Some(layout.align()),
        }
    }

    pub fn with_name(self, name: &CStr) -> MemBlockOptions<'_> {
        MemBlockOptions { name, ..self }
    }

    pub fn with_memory_partition(mut self, mem_partition: MemPartition) -> Self {
        self.mem_partition = mem_partition;
        self
    }

    pub fn alloc_mut(self) -> SceResult<MemBlockUninitMut> {
        unsafe { MemBlockUninitMut::from_raw(self.alloc_mut_raw()?) }
    }

    pub fn alloc_mut_raw(self) -> SceResult<MemBlockRaw> {
        let mut opt = self.alignment.map(|alignment| SceKernelAllocMemBlockOpt {
            size: 0x14,
            attr: SCE_KERNEL_ALLOC_MEMBLOCK_ATTR_HAS_ALIGNMENT,
            alignment: alignment as u32,
            uidBaseBlock: 0,
            strBaseBlockName: ptr::null(),
            flags: 0,
            reserved: unsafe { mem::zeroed() },
        });
        Ok(MemBlockRaw {
            uid: sce_result_uid_from_code(unsafe {
                sceKernelAllocMemBlock(
                    self.name.as_ptr(),
                    self.mem_partition.user_rw_type(),
                    self.size as u32,
                    opt.as_mut()
                        .map_or_else(ptr::null_mut, |r| r as *mut SceKernelAllocMemBlockOpt),
                )
            })?,
            len: self.size,
            mem_partition: self.mem_partition,
        })
    }
}

#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemPartition {
    #[default]
    Main,
    Cdram,
}

impl MemPartition {
    pub fn user_rw_type(self) -> SceKernelMemBlockType {
        match self {
            MemPartition::Main => SCE_KERNEL_MEMBLOCK_TYPE_USER_MAIN_RW,
            MemPartition::Cdram => SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
        }
    }

    pub const fn page_size(self) -> usize {
        match self {
            MemPartition::Main => 0x1000,
            MemPartition::Cdram => 0x40000,
        }
    }
}
