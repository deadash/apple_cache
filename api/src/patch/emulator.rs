use unicorn_engine::{Unicorn, unicorn_const::Permission, RegisterX86};

use crate::emu::{emu_map, emu_readp, emu_writep, as_u8_slice, emu_get_param, emu_set_param};

const SYSCALL_BASE: u64 = 0x200000000u64;
const SYSCALL_SIZE: u32 = 0x04;
const HEAP_STACK_BASE: u64 = 0x300000000u64;
const HEAP_STACK_SIZE: u32 = 0x30000;
const SYSCALL_MAX: u32 = 48;
const GLOBAL_EXIT_TAG:u64 = 0x1234123412341234u64;
const DATA_BASE: u64 = HEAP_STACK_BASE + HEAP_STACK_SIZE as u64;
const DATA_SIZE: u32 = 0x3000;
const STACK_CHK_GUARD: u64 = 0x1234567812345678;
const PARAM_BASE: u64 = DATA_BASE + DATA_SIZE as u64;
const PARAM_SIZE: u32 = 0x3000;

const MALLOC_BASE: u64 = PARAM_BASE + PARAM_SIZE as u64;
const MALLOC_SIZE: u32 = 0x3000;

// static SYSCALL_TAB: Vec<usize> = vec![
//     emulator_exit as usize,
    
// ];

pub const fn data_offset(offset: u32) -> u64
{
    DATA_BASE + offset as u64 * 8
}

fn not_implement<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    println!("+ not implement. {:x}", from);
    uc.reg_write(RegisterX86::RAX, 0).unwrap();
}

fn emulator_exit<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    println!("+ token: {:x}", from);
    uc.emu_stop().unwrap();
}

fn emulator_malloc<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let size: u64 = emu_get_param(uc, 0).unwrap();
    println!("+ malloc: {} from {:x}", size, from);
    let rax = match from {
        0x1000df097 => { MALLOC_BASE } // 512
        0x1000c82b2 => { MALLOC_BASE + 512 } // 832
        0x100114999 => { MALLOC_BASE + 512 + 832 } // 11
        _ => todo!()
    };
    uc.reg_write(RegisterX86::RAX, rax).unwrap();
}

fn emulator_free<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let ptr: u64 = emu_get_param(uc, 0).unwrap();
    println!("+ free: {:x}", ptr);
}

fn emulator_memcpy<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    // TODO: FIXME
    let src = emu_get_param(uc, 1).unwrap();
    uc.reg_write(RegisterX86::RAX, src).unwrap();
}

fn emulator_memset<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    todo!()
}

fn emulator_time<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let c = emu_get_param(uc, 0).unwrap();
    if c != 0 {
        todo!()
    }
    let tm = unsafe {
        libc::time(0 as _) as u64
    };
    uc.reg_write(RegisterX86::RAX, tm).unwrap();
}

fn emulator_pthread_once<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    // just call it
    let c = emu_get_param(uc, 1).unwrap();
    let rip = uc.reg_read(RegisterX86::RIP).unwrap();
    let rsp = uc.reg_read(RegisterX86::RSP).unwrap() - 8;
    uc.reg_write(RegisterX86::RSP, rsp).unwrap();
    emu_writep(uc, rsp, rip).unwrap();
    uc.set_pc(c).unwrap();
}

fn emulator_arc4random<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let rax = unsafe {
        libc::rand() as u64
    };
    uc.reg_write(RegisterX86::RAX, rax).unwrap();
}

fn emulator_sysctlbyname<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    // TODO: FIXME
    uc.reg_write(RegisterX86::RAX, 0).unwrap();
}

fn emulator_release<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    todo!()
}

fn emulator_skip_1<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    uc.reg_write(RegisterX86::RAX, 1).unwrap();
}

fn emulator_skip_0<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    uc.reg_write(RegisterX86::RAX, 0).unwrap();
}

fn emulator_create_with_cstr<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    todo!()
}

fn emulator_io_reg<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    todo!()
}

fn emulator_cf_<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    println!("+ cf not implement. {:x}", from);
    uc.emu_stop().unwrap();
}

fn emulator_syscall<'a>(uc: &mut Unicorn<'a, ()>)
{
    let rip = uc.reg_read(RegisterX86::RIP).unwrap();
    let rsp = uc.reg_read(RegisterX86::RSP).unwrap();
    let _return_address = emu_readp(uc, rsp).unwrap();
    let col = ((rip - SYSCALL_BASE - 1) / 4) as u32;
    let func = match col
    {
        0 => { emulator_exit }
        1 => { emulator_malloc }
        2 => { emulator_free }
        3 => { emulator_memcpy }
        4 => { emulator_memset }
        5 => { emulator_time }
        6 => { emulator_pthread_once }
        7 => { emulator_arc4random }
        8 => { emulator_sysctlbyname }
        9 | 21 => { emulator_release }
        10 | 23 | 29 => { emulator_skip_1 }
        11 => { emulator_create_with_cstr }
        12 => { emulator_io_reg }
        13 .. 20 => { emulator_cf_ }
        24 | 28 => { emulator_skip_0 }
        25 => { emulator_skip_0 } // TODO: _IOServiceGetMatchingServices
        _ => { not_implement }
    };

    func(uc, _return_address);
}

pub fn call_0<'a>(uc: &mut Unicorn<'a, ()>, cert: &[u8])
{
    let mut stack_top = HEAP_STACK_BASE + HEAP_STACK_SIZE as u64;
    stack_top -= 8;
    emu_writep(uc, stack_top, GLOBAL_EXIT_TAG).unwrap();
    stack_top -= 8;
    emu_writep(uc, stack_top, SYSCALL_BASE).unwrap();
    uc.reg_write(RegisterX86::RSP, stack_top).unwrap();
    // param
    if let Err(e) = emu_map(uc, PARAM_BASE, PARAM_SIZE as usize, Permission::READ | Permission::WRITE) {
        println!("+ failed: {:x} {:?}", PARAM_BASE, e);
    }
    // pass
    uc.mem_write(PARAM_BASE, &vec![0u8; 32]).unwrap();
    uc.mem_write(PARAM_BASE + 32, cert).unwrap();
    emu_set_param(uc, 0, PARAM_BASE + 32).unwrap();
    emu_set_param(uc, 1, cert.len() as u64).unwrap();
    emu_set_param(uc, 2, PARAM_BASE).unwrap();
    emu_set_param(uc, 3, PARAM_BASE + 8).unwrap();
    emu_set_param(uc, 4, PARAM_BASE + 16).unwrap();

    uc.emu_start(0x1000d2f30, 0, 0, 0).unwrap();
    let result = uc.reg_read(RegisterX86::RAX).unwrap();
    println!("+ result: {:x}", result);
}

fn debug_code<'a>(uc: &mut Unicorn<'a, ()>, address: u64, size: u32)
{
    // println!("+ code: {:x} RDX:{:x}", address, uc.reg_read(RegisterX86::RDX).unwrap());
}

pub fn regiser_init<'a>(uc: &mut Unicorn<'a, ()>)
{
    println!("+ start register init.");
    let size = (SYSCALL_SIZE * SYSCALL_MAX) as usize;
    if let Err(e) = emu_map(uc, SYSCALL_BASE, size, Permission::ALL) {
        println!("+ failed: {:x} {:?}", SYSCALL_BASE, e);
    }

    // register heap
    if let Err(e) = emu_map(uc, HEAP_STACK_BASE, HEAP_STACK_SIZE as usize, Permission::READ | Permission::WRITE) {
        println!("+ failed: {:x} {:?}", HEAP_STACK_BASE, e);
    }

    let code:Vec<u8> = vec![0x90, 0x0f, 0x05, 0xc3];
    let mut codes: Vec<u8> = Vec::new();
    for _ in 0 .. SYSCALL_MAX {
        codes.extend(code.iter());
    }
    if let Err(e) = uc.mem_write(SYSCALL_BASE, &codes) {
        println!("+ failed: {:?}", e);
    }

    // register callback
    if let Err(e) = uc.add_insn_sys_hook(unicorn_engine::InsnSysX86::SYSCALL, SYSCALL_BASE, SYSCALL_BASE + size as u64, emulator_syscall) {
        println!("+ failed: {:?}", e);
    }

    // init .data
    if let Err(e) = emu_map(uc, DATA_BASE, DATA_SIZE as usize, Permission::READ | Permission::WRITE) {
        println!("+ failed: {:x} {:?}", DATA_BASE, e);
    }

    // TODO: 大小端问题
    let mut data: Vec<u64> = Vec::new();
    data.push(STACK_CHK_GUARD);
    data.push(data_offset(0));
    for i in 0 .. 10 {
        data.push(data_offset(i + 2));
    }
    if let Err(e) = uc.mem_write(DATA_BASE, as_u8_slice(&data)) {
        println!("+ failed: {:?}", e);
    }

    if let Err(e) = uc.add_code_hook(1, 0, debug_code) {
        println!("+ failed: {:?}", e);
    }

    // TODO: malloc
    if let Err(e) = emu_map(uc, MALLOC_BASE, MALLOC_SIZE as usize, Permission::READ | Permission::WRITE) {
        println!("+ failed: {:x} {:?}", MALLOC_BASE, e);
    }

    println!("+ register init done.");
}

pub fn register_fn(_dylib: &str, name: &str) -> u64
{
    // data
    let data = match name {
        "____chkstk_darwin" => Some(0),
        "___CFConstantStringClassReference" => Some(data_offset(0)),
        "___stack_chk_guard" => Some(data_offset(1)),
        "_kIOMasterPortDefault" => Some(data_offset(2)),
        "_kCFAllocatorDefault" => Some(data_offset(3)),
        "_kCFAllocatorNull" => Some(data_offset(4)),
        "_kCFTypeDictionaryKeyCallBacks" => Some(data_offset(5)),
        "_kCFTypeDictionaryValueCallBacks" => Some(data_offset(6)),
        "_kCFBooleanTrue" => Some(data_offset(7)),
        "_kCFBooleanFalse" => Some(data_offset(8)),
        "___kCFBooleanFalse" => Some(data_offset(9)),
        "___kCFBooleanTrue" => Some(data_offset(10)),
        _ => None,
    };
    if let Some(data) = data {
        return data;
    }
    let col = match name
    {
        "_malloc" => 1,
        "_free" => 2,
        // "_stack_chk_fail" => 5,
        "___memcpy_chk" => 3,
        "___memset_chk" => 4,
        "_time" => 5,
        "_pthread_once" => 6,
        "_arc4random" => 7,
        // start to patch
        "_sysctlbyname" => 8,
        "_IOObjectRelease" => 9,
        "_IORegistryEntryFromPath" => 10,
        "_CFStringCreateWithCStringNoCopy" => 11,
        "_IORegistryEntryCreateCFProperty" => 12,
        "_CFGetTypeID" => 13,
        "_CFStringGetTypeID" => 14,
        "_CFStringGetLength" => 15,
        "_CFStringGetMaximumSizeForEncoding" => 16,
        "_CFStringGetCString" => 17,
        "_CFDataGetTypeID" => 18,
        "_CFDataGetLength" => 19,
        "_CFDataGetBytes" => 20,
        "_CFRelease" => 21,
        "_IOServiceMatching" => 22,
        // 网卡
        "_CFDictionaryCreateMutable" => 23,
        "_CFDictionarySetValue" => 24,
        "_IOServiceGetMatchingServices" => 25,
        "_IOIteratorReset" => 26,
        "_IOIteratorNext" => 27,
        "_IORegistryEntryGetParentEntry" => 28,
        "_CFStringGetSystemEncoding" => 29,
        "___CFConstantStringClassReference" => 30,
        _ => SYSCALL_MAX - 1,
    } as u32;

    SYSCALL_BASE + (SYSCALL_SIZE * col) as u64
}