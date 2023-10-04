use core::{
    ffi::c_void,
    mem, ptr,
    sync::atomic::{self, AtomicBool},
};

use crate::{
    error::sce_result_unit_from_code,
    sysmem::{MemBlockMut, MemBlockOptions, MemPartition},
    SceError, SceResult,
};

#[derive(Debug)]
pub struct Display {
    current: Option<Framebuf>,
}

static DISPLAY_TAKEN: AtomicBool = AtomicBool::new(false);

impl Display {
    pub fn take() -> Option<Self> {
        let res = DISPLAY_TAKEN.compare_exchange(
            false,
            true,
            atomic::Ordering::Acquire,
            atomic::Ordering::Relaxed,
        );
        match res {
            Ok(_) => Some(Display { current: None }),
            Err(_) => None,
        }
    }

    pub fn is_set(&self) -> bool {
        self.current.is_some()
    }

    pub fn replace_framebuf(
        &mut self,
        fb: Framebuf,
    ) -> Result<Option<Framebuf>, (SceError, Framebuf)> {
        let desc = fb.to_sce();
        let res = sce_result_unit_from_code(unsafe {
            vitasdk_sys::sceDisplaySetFrameBuf(&desc, vitasdk_sys::SCE_DISPLAY_SETBUF_NEXTFRAME)
        });
        match res {
            Ok(()) => Ok(self.current.replace(fb)),
            Err(e) => Err((e, fb)),
        }
    }

    pub fn take_framebuf(&mut self) -> SceResult<Option<Framebuf>> {
        sce_result_unit_from_code(unsafe {
            vitasdk_sys::sceDisplaySetFrameBuf(
                ptr::null(),
                vitasdk_sys::SCE_DISPLAY_SETBUF_NEXTFRAME,
            )
        })?;
        Ok(self.current.take())
    }

    pub fn wait_set_framebuf(&self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { vitasdk_sys::sceDisplayWaitSetFrameBuf() })
    }

    pub fn wait_vblank_start(&self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { vitasdk_sys::sceDisplayWaitVblankStart() })
    }

    pub fn wait_vblank_start_multi(&self, vcount: u32) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { vitasdk_sys::sceDisplayWaitVblankStartMulti(vcount) })
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        let _ = self.take_framebuf();
        let _ = DISPLAY_TAKEN.compare_exchange(
            true,
            false,
            atomic::Ordering::Release,
            atomic::Ordering::Relaxed,
        );
    }
}

#[derive(Debug)]
pub struct Framebuf {
    pub memblock: MemBlockMut,
    pub desc: FramebufDesc,
}

impl Framebuf {
    pub fn native() -> SceResult<Self> {
        FramebufDesc::NATIVE.alloc_mut_zeroed()
    }

    pub fn new(memblock: MemBlockMut, desc: FramebufDesc) -> Self {
        Framebuf { memblock, desc }
    }

    fn to_sce(&self) -> vitasdk_sys::SceDisplayFrameBuf {
        self.desc.to_sce(self.memblock.as_mut_ptr().cast())
    }
}

/// Framebuffer descriptor
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FramebufDesc {
    pub width: u32,
    pub pitch: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
}

impl Default for FramebufDesc {
    fn default() -> Self {
        FramebufDesc::NATIVE
    }
}

impl FramebufDesc {
    pub const fn new(width: u32, height: u32) -> Self {
        FramebufDesc {
            pitch: min_pitch(width),
            pixel_format: PixelFormat::A8B8G8R8,
            width,
            height,
        }
    }

    pub const NATIVE: Self = FramebufDesc::W960H544;
    pub const W480H272: Self = FramebufDesc::new(480, 272);
    pub const W640H368: Self = FramebufDesc::new(640, 368);
    pub const W704H488: Self = FramebufDesc::new(704, 488);
    pub const W720H408: Self = FramebufDesc::new(720, 408);
    pub const W960H544: Self = FramebufDesc::new(960, 544);

    pub fn bytes_needed(&self) -> usize {
        match self.pixel_format {
            PixelFormat::A8B8G8R8 | PixelFormat::A2B10G10R10 => 4_usize,
        }
        .checked_mul(self.pitch as usize)
        .and_then(|b| b.checked_mul(self.height as usize))
        .and_then(|b| {
            let page_size = MemPartition::Cdram.page_size();
            let page_mask = page_size - 1;
            (b & !page_mask).checked_add(if b & page_mask != 0 { page_size } else { 0 })
        })
        .expect("Instance of FramebufDesc requires too many bytes")
    }

    pub fn alloc_mut_zeroed(self) -> SceResult<Framebuf> {
        let memblock = MemBlockOptions::from_size(self.bytes_needed())
            .with_memory_partition(MemPartition::Cdram)
            .alloc_mut()?;
        #[cfg(feature = "dmac")]
        let memblock = memblock.dmac_fill_init(0)?;
        #[cfg(not(feature = "dmac"))]
        let memblock = memblock.fill_init(0);
        Ok(Framebuf::new(memblock, self))
    }

    fn to_sce(&self, base: *mut c_void) -> vitasdk_sys::SceDisplayFrameBuf {
        vitasdk_sys::SceDisplayFrameBuf {
            size: mem::size_of::<vitasdk_sys::SceDisplayFrameBuf>() as u32,
            base,
            pitch: self.pitch,
            pixelformat: self.pixel_format as u32,
            width: self.width,
            height: self.height,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn with_pitch(mut self, pitch: u32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn with_pixel_format(mut self, pixel_format: PixelFormat) -> Self {
        self.pixel_format = pixel_format;
        self
    }
}

#[non_exhaustive]
#[repr(u32)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    #[default]
    A8B8G8R8 = vitasdk_sys::SCE_DISPLAY_PIXELFORMAT_A8B8G8R8,
    A2B10G10R10 = vitasdk_sys::SCE_DISPLAY_PIXELFORMAT_A2B10G10R10,
}

#[track_caller]
pub const fn min_pitch(width: u32) -> u32 {
    // Pitch has to be multiple of 64
    match (width & !63).checked_add(if width & 63 != 0 { 64 } else { 0 }) {
        Some(p) => p,
        None => panic!("Width is too large"),
    }
}
