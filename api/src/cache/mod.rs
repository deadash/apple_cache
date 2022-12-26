

#[cfg(feature = "emu")]
mod emulator;
#[cfg(feature = "emu")]
pub use self::emulator::*;

#[cfg(not(feature = "emu"))]
mod cache;
#[cfg(not(feature = "emu"))]
pub use self::cache::*;