use crate::{macho::MachoLoader, patch};
use anyhow::{Result, Context};
use rust_embed::RustEmbed;
use bincode::config;
use std::ffi::c_void;

#[derive(RustEmbed)]
#[folder = "../blob"]
#[include = "AssetCache~.x64"]
#[include = "cert.cer"]
#[include = "reloc.bin"]
struct Asset;

pub struct Cache
{
    loader: MachoLoader,
}

impl Cache
{
    pub fn new() -> Result<Cache>
    {
        // read from memory.
        let file = Asset::get("AssetCache~.x64").context("")?;
        let loader = MachoLoader::new(&file.data, patch::register_fn)?;
        // TODO: 等待移除
        let config = config::standard();
        let file = Asset::get("reloc.bin").context("")?;
        let (relocs,_ ): (Vec<usize>, usize) = bincode::decode_from_slice(&file.data, config)?;
        unsafe { loader.fixup_relocs(&relocs) }
        Ok(Cache { loader })
    }

    pub fn offset(&self, addr: usize) -> usize
    {
        self.loader.get_offset(addr)
    }

    pub fn create(&self, _cert: Option<&[u8]>) -> Result<(usize, Vec<u8>)>
    {
        let cert = Asset::get("cert.cer").context("")?;
        let pcert = &cert.data;
        let func: extern "sysv64" fn(*const u8, usize, *mut usize, *mut *const c_void, *mut i32) -> i32 = 
            unsafe { core::mem::transmute(self.offset(0x1000d2f30))};

        let mut ctx: usize = 0;
        let mut data: *const c_void = 0 as _;
        let mut data_len: i32 = 0;
        let result = func(pcert.as_ptr(), pcert.len(), &mut ctx, &mut data, &mut data_len);

        if result != 0 {
            unreachable!()
        } else {
            let data = unsafe { std::slice::from_raw_parts(data as *const u8, data_len as usize).to_owned() };
            Ok((ctx, data))
        }
    }

    pub fn obtain(&self, ctx: usize, session: &[u8]) -> Result<()>
    {
        let func: extern "sysv64" fn(usize, *const c_void, i32) -> i32 =
            unsafe { core::mem::transmute(self.offset(0x100125a50))};
        let result = func(ctx, session.as_ptr() as _, session.len() as i32);

        if result != 0 {
            unreachable!()
        } else {
            Ok(())
        }
    }

    pub fn sign(&self, ctx: usize, data: &[u8]) -> Result<Vec<u8>>
    {
        let func: extern "sysv64" fn(usize, *const c_void, i32, *mut *const c_void, *mut i32) -> i32 =
            unsafe { core::mem::transmute(self.offset(0x1000c5860))};
        let mut ret_data: *const c_void = 0 as _;
        let mut data_len: i32 = 0;
        let result = func(ctx, data.as_ptr() as _, data.len() as i32, &mut ret_data, &mut data_len);

        if result != 0 {
            unreachable!()
        } else {
            let data = unsafe { std::slice::from_raw_parts(ret_data as *const u8, data_len as usize).to_owned() };
            Ok(data)
        }
    }
}