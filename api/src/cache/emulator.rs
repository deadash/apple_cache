use crate::{macho::MachoLoader, patch};
use anyhow::{Result, Context};
use rust_embed::RustEmbed;
use unicorn_engine::{Unicorn, unicorn_const::{Arch, Mode}};

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
        let file = Asset::get("AssetCache~.x64").context("")?;
        let data = file.data;
        let mut uc = Unicorn::new(Arch::X86, Mode::LITTLE_ENDIAN | Mode::MODE_64).unwrap();
        let loader = MachoLoader::new(&data, &mut uc, patch::register_fn)?;
        // install function hook
        patch::regiser_init(&mut uc);
        Ok(Cache { loader, uc })
    }

    pub fn offset(&self, addr: u64) -> u64
    {
        self.loader.get_offset(addr)
    }

    pub fn create(&mut self, _cert: Option<&[u8]>) -> Result<(u64, Vec<u8>)>
    {
        let cert = Asset::get("cert.cer").context("")?;
        let pcert = &cert.data;
        patch::call_0(&mut self.uc, &pcert)
    }

    pub fn obtain(&mut self, ctx: u64, session: &[u8]) -> Result<()>
    {
        patch::call_1(&mut self.uc, ctx, session)
    }

    pub fn sign(&mut self, ctx: u64, data: &[u8]) -> Result<Vec<u8>>
    {
        patch::call_2(&mut self.uc, ctx, data)
    }
}