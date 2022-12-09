use crate::{macho::MachoLoader, patch};
use anyhow::{Result, Context};
use rust_embed::RustEmbed;
use unicorn_engine::{Unicorn, unicorn_const::{Arch, Mode}};
use std::{ffi::c_void, fs};

#[derive(RustEmbed)]
#[folder = "../blob"]
#[include = "AssetCache~.x64"]
#[include = "cert.cer"]
struct Asset;

pub struct Cache<'a>
{
    loader: MachoLoader,
    uc: Unicorn<'a, ()>,
}

impl <'a>Cache<'a>
{
    pub fn new() -> Result<Cache<'a>>
    {
        // read from memory.
        // let file = Asset::get("AssetCache~.x64").context("")?;
        // let data = file.data;
        let file = fs::read("blob/AssetCache~.x64")?;
        let mut uc = Unicorn::new(Arch::X86, Mode::LITTLE_ENDIAN | Mode::MODE_64).unwrap();
        let loader = MachoLoader::new(&file, &mut uc, patch::register_fn)?;
        // install function hook
        patch::regiser_init(&mut uc);
        Ok(Cache { loader, uc })
    }

    pub fn offset(&self, addr: u64) -> u64
    {
        self.loader.get_offset(addr)
    }

    pub fn create(&mut self, _cert: Option<&[u8]>) -> Result<(usize, Vec<u8>)>
    {
        let cert = Asset::get("cert.cer").context("")?;
        let pcert = &cert.data;
        // test
        patch::call_0(&mut self.uc, &pcert);
        todo!()
    }

    pub fn obtain(&self, ctx: usize, session: &[u8]) -> Result<()>
    {
        todo!()
    }

    pub fn sign(&self, ctx: usize, data: &[u8]) -> Result<Vec<u8>>
    {
        todo!()
    }
}