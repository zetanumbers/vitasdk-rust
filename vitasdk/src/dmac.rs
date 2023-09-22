use std::mem;

use crate::{error::sce_result_unit_from_code, SceResult};

#[doc(alias = "memcpy")]
pub unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) -> SceResult<()> {
    sce_result_unit_from_code(unsafe {
        vitasdk_sys::sceDmacMemcpy(dst.cast(), src.cast(), (count * mem::size_of::<T>()) as u32)
    })
}

#[doc(alias = "memset")]
pub unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize) -> SceResult<()> {
    sce_result_unit_from_code(unsafe {
        vitasdk_sys::sceDmacMemset(dst.cast(), val.into(), (count * mem::size_of::<T>()) as u32)
    })
}

#[doc(alias = "memcpy")]
pub trait DmacSliceCopyExt: dmac_slice_copy_ext_private::Sealed {
    fn dmac_copy_from_slice(&mut self, other: &Self) -> SceResult<()>;
}

impl<T> DmacSliceCopyExt for [T]
where
    T: Copy,
{
    #[track_caller]
    fn dmac_copy_from_slice(&mut self, src: &[T]) -> SceResult<()> {
        // The panic code path was put into a cold function to not bloat the
        // call site.
        #[inline(never)]
        #[cold]
        #[track_caller]
        fn len_mismatch_fail(dst_len: usize, src_len: usize) -> ! {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                src_len, dst_len,
            );
        }

        if self.len() != src.len() {
            len_mismatch_fail(self.len(), src.len());
        }

        // SAFETY: `self` is valid for `self.len()` elements by definition, and `src` was
        // checked to have the same length. The slices cannot overlap because
        // mutable references are exclusive.
        unsafe {
            let count = self.len();
            copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), count)
        }
    }
}

mod dmac_slice_copy_ext_private {
    pub trait Sealed {}
    impl<T: Copy> Sealed for [T] {}
}

pub trait DmacSliceFillExt: slice_fill_ext_private::Sealed {
    fn dmac_fill(&mut self, value: u8) -> SceResult<()>;
}

impl DmacSliceFillExt for [u8] {
    #[track_caller]
    fn dmac_fill(&mut self, value: u8) -> SceResult<()> {
        unsafe {
            let count = self.len();
            write_bytes(self.as_mut_ptr(), value, count)
        }
    }
}

impl DmacSliceFillExt for [mem::MaybeUninit<u8>] {
    #[track_caller]
    fn dmac_fill(&mut self, value: u8) -> SceResult<()> {
        unsafe {
            let count = self.len();
            write_bytes(self.as_mut_ptr(), value, count)
        }
    }
}

mod slice_fill_ext_private {
    pub trait Sealed {}
    impl Sealed for [u8] {}
    impl Sealed for [super::mem::MaybeUninit<u8>] {}
}
