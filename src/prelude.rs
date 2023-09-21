//! The basics to get you going with sarge.

pub use crate::{tag, ArgumentParser};

#[cfg(feature = "macros")]
pub use crate::Arguments;
#[cfg(feature = "macros")]
pub use sarge_macros::Arguments;
