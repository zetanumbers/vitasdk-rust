use core::num::NonZeroI32;

use crate::types::Uid;

pub type SceResult<T> = Result<T, SceError>;

// TODO: rename these to shorter names
#[track_caller]
pub fn sce_result_unit_from_code(code: i32) -> SceResult<()> {
    if let Some(code) = NonZeroI32::new(code) {
        debug_assert!(code.get() < 0, "return code is positive: {code}");
        Err(SceError::from_raw_error(code))
    } else {
        Ok(())
    }
}

#[track_caller]
pub fn sce_result_uid_from_code(code: i32) -> SceResult<Uid> {
    let uid = Uid::new(code).expect("sce function returned zero uid");
    if uid.get() < 0 {
        Err(SceError::from_raw_error(uid))
    } else {
        Ok(uid)
    }
}

// TODO: Add consts
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SceError(NonZeroI32);

impl SceError {
    pub fn from_raw_error(code: NonZeroI32) -> SceError {
        SceError(code)
    }

    pub fn code(&self) -> NonZeroI32 {
        self.0
    }
}

impl core::fmt::Display for SceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SceError with code {:x}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SceError {}
