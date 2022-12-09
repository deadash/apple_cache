#[cfg(feature = "emu")]
mod emulator;
#[cfg(feature = "emu")]
pub use self::emulator::*;

#[cfg(not(feature = "emu"))]
mod patch;
#[cfg(not(feature = "emu"))]
pub use self::patch::*;

// TODO: use osx