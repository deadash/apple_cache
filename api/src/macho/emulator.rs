use anyhow::Result;
use goblin::mach;
use unicorn_engine::{Unicorn, unicorn_const::Permission};

use crate::emu::{emu_map, emu_writep};

pub struct MachoLoader
{
    image_base: u64,
    base_addr: u64,
}

pub type SymbolCallback = fn (dylib: &str, name: &str) -> u64;

impl MachoLoader
{
    pub fn new(file: &[u8], uc: &mut Unicorn<()>, callback: SymbolCallback) -> Result<MachoLoader>
    {
        let macho = mach::Mach::parse(&file)?;
        let loader = match macho {
            mach::Mach::Fat(_bin) => {
                // TODO: use Fat.
                log::error!("+ does not support fat yet");
                unreachable!()
            }
            mach::Mach::Binary(bin) => {
                let mut loader = MachoLoader { image_base: 0x100000000u64, base_addr: 0x100000000u64 };
                loader.init(&bin, &file, uc, callback)?;
                loader
            }
        };
        Ok(loader)
    }
    pub fn get_offset(&self, addr: u64) -> u64
    {
        addr
    }

    fn init(&mut self, macho: &mach::MachO, file: &[u8], uc: &mut Unicorn<()>, callback: SymbolCallback) -> Result<()>
    {
        // 计算镜像大小
        let mut image_end = self.image_base;
        for sec in &macho.segments {
            if sec.filesize == 0 {
                continue;
            }
            let end = sec.vmaddr + sec.filesize;
            if end > image_end {
                image_end = end;
            }
        }
        let image_size = image_end - self.image_base;
        log::info!("+ alloc image size {:x}", image_size);
        // 映射内存
        if let Err(e) = emu_map(uc, self.base_addr, image_size as usize, Permission::ALL) {
            log::error!("+ failed map: {:x} {:?}", self.base_addr, e);
        }

        // 写入数据
        for sec in &macho.segments {
            // TODO: padding
            if let Err(e) = uc.mem_write(sec.vmaddr, &file[sec.fileoff as usize .. (sec.fileoff + sec.filesize) as usize]) {
                log::error!("+ failed write: {:x} {:?}", sec.vmaddr, e);
            }
        }
        log::info!("+ write done.");
    
        // 导入符号
        for imp in macho.imports()? {
            // TODO: may use
            if imp.is_lazy {
                continue;
            }
            let new_addr = (callback(imp.dylib, imp.name) as i64 + imp.addend) as u64;
            let addr = self.get_offset(imp.address);
            if let Err(e) = emu_writep(uc, addr, new_addr) {
                log::error!("+ failed write: {:x} {:x?}, {:?}", addr, new_addr, e);
            }
        }
        log::info!("+ done.");
        Ok(())
    }
}