use std::ffi::c_void;
use anyhow::Result;
use goblin::mach;

#[derive(Default)]
pub struct MachoLoader
{
    image_base: usize,
    base_addr: usize, 
    page: Option<region::Allocation>,
}

pub type SymbolCallback = fn (dylib: &str, name: &str) -> usize;

impl MachoLoader
{
    pub fn new(file: &[u8], callback: SymbolCallback) -> Result<MachoLoader>
    {
        let macho = mach::Mach::parse(&file)?;
        let loader = match macho {
            mach::Mach::Fat(_bin) => {
                // TODO: use Fat.
                log::error!("+ does not support fat yet");
                unreachable!()
            }
            mach::Mach::Binary(bin) => {
                let mut loader = MachoLoader { .. Default::default() };
                loader.init(&bin, file, callback)?;
                loader
            }
        };
        Ok(loader)
    }
    pub fn get_offset(&self, addr: usize) -> usize
    {
        if let Some(page) = &self.page {
            page.as_ptr() as *const c_void as usize - self.image_base + addr
        } else {
            0
        }
    }

    fn init(&mut self, macho: &mach::MachO, file: &[u8], callback: SymbolCallback) -> Result<()>
    {
        // TODO: read from macho
        self.image_base = 0x100000000usize;
        // 计算镜像大小
        let mut image_end = self.image_base;
        for sec in &macho.segments {
            if sec.filesize == 0 {
                continue;
            }
            let end = (sec.vmaddr + sec.filesize) as usize;
            if end > image_end {
                image_end = end;
            }
        }
        let image_size = image_end - self.image_base;
        log::info!("+ alloc image size {:x}", image_size);
        // 映射内存
        let page = region::alloc(image_size, region::Protection::READ_WRITE_EXECUTE)?;
        self.base_addr = page.as_ptr() as *const c_void as usize;
        self.page = Some(page);
    
        // 写入数据
        for sec in &macho.segments {
            // TODO: padding
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (file.as_ptr() as *mut u8).offset(sec.fileoff as isize), self.get_offset(sec.vmaddr as usize) as _, sec.filesize as usize
                );
            }
        }
    
        // 导入符号
        for imp in macho.imports()? {
            // TODO: may use
            if imp.is_lazy {
                continue;
            }
            let new_addr = (callback(imp.dylib, imp.name) as i64 + imp.addend) as usize;
            let addr = self.get_offset(imp.address as usize);
            unsafe { core::ptr::write(addr as *mut usize, new_addr) };
        }
        Ok(())
    }

    // TODO: Removed after repairing the relocation table
    pub unsafe fn fixup_relocs(&self, relocs: &[usize])
    {
        for v in relocs {
            let ptr: *mut usize = self.get_offset(v + self.image_base) as _;
            if *ptr > self.image_base {
                *ptr -= self.image_base;
            }
            *ptr += self.base_addr;
        }
    }
}