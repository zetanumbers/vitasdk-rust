#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::std_instead_of_alloc, clippy::std_instead_of_core)]

#[cfg(feature = "display")]
#[cfg_attr(docsrs, doc(cfg(feature = "display")))]
pub mod display;
#[cfg(feature = "dmac")]
#[cfg_attr(docsrs, doc(cfg(feature = "dmac")))]
pub mod dmac;
pub mod error;
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub mod http;
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;
#[cfg(feature = "sysmem")]
#[cfg_attr(docsrs, doc(cfg(feature = "sysmem")))]
pub mod sysmem;
#[cfg(feature = "sysmodule")]
#[cfg_attr(docsrs, doc(cfg(feature = "sysmodule")))]
pub mod sysmodule;
mod types;

pub use error::{SceError, SceResult};
pub use types::*;
