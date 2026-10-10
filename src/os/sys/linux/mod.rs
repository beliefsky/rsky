mod general;
#[cfg(target_arch = "x86_64")]
mod x86_64;

pub use general::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
