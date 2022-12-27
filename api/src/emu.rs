use core::slice::memchr::memchr;
use std::ffi::CString;

use unicorn_engine::{Unicorn, unicorn_const::{Permission, uc_error}, RegisterX86};
use anyhow::Result;

fn align(a: u64, v: u64) -> u64
{
    ((a - 1) & !(v - 1)) + v
}

fn alignb(a: u64, v: u64) -> u64
{
    a & !(v - 1)
}

pub fn emu_map<'a>(uc: &mut Unicorn<'a, ()>,
    address: u64,
    size: libc::size_t,
    perms: Permission) -> Result<(), uc_error>
{
    let start = alignb(address, 0x1000u64);
    let end = align(address + size as u64, 0x1000u64);
    uc.mem_map(start, (end - start) as usize, perms)
}

pub fn emu_writep<'a>(uc: &mut Unicorn<'a, ()>, address: u64, ptr: u64)
    -> Result<(), uc_error>
{
    let ptr = ptr.to_le();
    if let Err(e) = uc.mem_write(address, as_u8_slice(&ptr)) {
        return Err(e)
    } else {
        Ok(())
    }
}

pub fn emu_readp<'a>(uc: &mut Unicorn<'a, ()>, address: u64)
    -> Result<u64, uc_error>
{
    let mut ret: u64 = 0;
    if let Err(e) = uc.mem_read(address, as_u8_slice_mut(&mut ret)) {
        return Err(e)
    } else {
        Ok(u64::from_le(ret))
    }
}

pub fn emu_reads<'a>(uc: &mut Unicorn<'a, ()>, address: u64)
    -> Result<String, uc_error>
{
    let mut v = vec![0u8; 1024];
    // TODO: use resize to get more.
    if let Err(e) = uc.mem_read(address, &mut v) {
        return Err(e)
    } else {
        let len = memchr(0, &v).unwrap_or(v.len());
        let s = unsafe { CString::from_vec_unchecked(v[..len].to_vec()) }.into_string().unwrap();
        Ok(s)
    }
}

pub fn emu_writes<'a>(uc: &mut Unicorn<'a, ()>, address: u64, s: String)
    -> Result<(), uc_error>
{
    emu_writev(uc, address, s.as_bytes())
}

pub fn emu_writev<'a>(uc: &mut Unicorn<'a, ()>, address: u64, v: &[u8])
    -> Result<(), uc_error>
{
    if let Err(e) = uc.mem_write(address, v) {
        return Err(e)
    } else {
        Ok(())
    }
}

pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::std::slice::from_raw_parts(
            (p as *const T) as *const u8,
            ::std::mem::size_of::<T>(),
        )
    }
}

pub fn as_u8_slice_mut<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe {
        ::std::slice::from_raw_parts_mut(
            (p as *mut T) as *mut u8,
            ::std::mem::size_of::<T>(),
        )
    }
}

pub fn emu_set_param<'a>(uc: &mut Unicorn<'a, ()>, idx: u32, ptr: u64) -> Result<(), uc_error>
{
    if idx < 6 {
        let regid = match idx {
            0 => RegisterX86::RDI,
            1 => RegisterX86::RSI,
            2 => RegisterX86::RDX,
            3 => RegisterX86::RCX,
            4 => RegisterX86::R8,
            5 => RegisterX86::R9,
            // TODO: error
            _ => RegisterX86::RAX,
        };
        uc.reg_write(regid, ptr)
    } else {
        match uc.reg_read(RegisterX86::RSP)
        {
            Ok(rsp) => {
                emu_writep(uc, rsp + 8 * (idx as u64 - 5), ptr)
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

pub fn emu_get_param<'a>(uc: &mut Unicorn<'a, ()>, idx: u32) -> Result<u64, uc_error>
{
    if idx < 6 {
        let regid = match idx {
            // TODO: error
            0 => RegisterX86::RDI,
            1 => RegisterX86::RSI,
            2 => RegisterX86::RDX,
            3 => RegisterX86::RCX,
            4 => RegisterX86::R8,
            5 => RegisterX86::R9,
            _ => RegisterX86::RAX,
        };
        uc.reg_read(regid)
    } else {
        match uc.reg_read(RegisterX86::RSP)
        {
            Ok(rsp) => {
                emu_readp(uc, rsp + 8 * (idx as u64 - 5))
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}