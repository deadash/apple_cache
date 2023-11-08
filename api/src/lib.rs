#![allow(non_snake_case)]
#![feature(exclusive_range_pattern)]
#![feature(slice_internals)]

use cache::Cache;

mod patch;
mod macho;
mod cache;
mod mac_serial;

#[cfg(feature = "emu")]
mod emu;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("failed. {0}!")]
    HandleFailed(String),
}

#[derive(Debug, Clone)]
pub struct CacheResult {
    ctx: u64,
    data: Vec<u8>,
}

type Result<T, E = CacheError> = std::result::Result<T, E>;

pub struct CacheApi {
    c: Cache,
}

impl CacheApi {
    pub fn new() -> Result<Self> {
        match Cache::new() {
            Ok(c) => Ok(Self { c }),
            Err(e) => Err(CacheError::HandleFailed(e.to_string()))
        }
    }

    pub fn create(&self, cert: Option<Vec<u8>>) -> Result<CacheResult, CacheError> {
        match self.c.create(Some(&cert.unwrap_or_default())) {
            Ok((ctx, data)) => {
                Ok(CacheResult { ctx: ctx as u64, data })
            },
            Err(e) => Err(CacheError::HandleFailed(e.to_string()))
        }
    }

    pub fn obtain(&self, ctx: u64, session: Vec<u8>) -> Result<(), CacheError> {
        match self.c.obtain(ctx as usize, &session) {
            Ok(_) => {
                Ok(())
            },
            Err(e) => Err(CacheError::HandleFailed(e.to_string()))
        }
    }

    pub fn sign(&self, ctx: u64, data: Vec<u8>) -> Result<Vec<u8>, CacheError> {
        match self.c.sign(ctx as usize, &data) {
            Ok(d) => {
                Ok(d)
            },
            Err(e) => Err(CacheError::HandleFailed(e.to_string()))
        }
    }
}

pub fn init_serial(serial: String) -> Result<()> {
    if let Err(e) = mac_serial::MacSerial::instance().init_from_json(&serial) {
        return Err(CacheError::HandleFailed(e.to_string()));
    }
    Ok(())
}

unsafe impl Send for CacheApi {}

uniffi::include_scaffolding!("lib");