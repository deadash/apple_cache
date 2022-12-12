use libc::printf;
use unicorn_engine::{Unicorn, unicorn_const::Permission, RegisterX86};
use anyhow::Result;
use crate::emu::{emu_map, emu_readp, emu_writep, as_u8_slice, emu_get_param, emu_set_param, emu_reads, as_u8_slice_mut};

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

static mut GLOBAL_M: u64 = 0;
static mut GLOBAL_V: Vec<(u8, Vec<u8>)> = Vec::new();

pub const fn data_offset(offset: u32) -> u64
{
    DATA_BASE + offset as u64 * 8
}

// ptr as str
fn cf_to_str<'a>(uc: &mut Unicorn<'a, ()>, ptr: u64) -> String
{
    let max = unsafe { GLOBAL_V.len() as u64 };
    if ptr <= max {
        unsafe {
            if let Some((_t, v)) = GLOBAL_V.get(ptr as usize - 1) {
                // default t == 1
                String::from_utf8(v.to_vec()).unwrap()
            } else {
                todo!()
            }
        }
    } else {
        let ptr = emu_readp(uc, ptr + 0x10).unwrap();
        let s = emu_reads(uc, ptr).unwrap();
        s
    }
}

// str as ptr
fn str_to_cf<'a>(uc: &mut Unicorn<'a, ()>, s: &str) -> u64
{
    unsafe {
        GLOBAL_V.push((1, s.as_bytes().to_vec()));
        GLOBAL_V.len() as u64
    }
}

//
fn cf_to_bytes<'a>(uc: &mut Unicorn<'a, ()>, ptr: u64) -> Vec<u8>
{
    unsafe {
        if let Some((_t, v)) = GLOBAL_V.get(ptr as usize - 1) {
            // default t == 2
            v.to_vec()
        } else {
            todo!()
        }
    }
}

// data as ptr
fn vec_to_cf<'a>(uc: &mut Unicorn<'a, ()>, v: Vec<u8>) -> u64
{
    unsafe {
        GLOBAL_V.push((2, v));
        GLOBAL_V.len() as u64
    }
}

// cf
fn cf_str_id<'a>(uc: &mut Unicorn<'a, ()>, from: u64) { uc.reg_write(RegisterX86::RAX, 1).unwrap(); }
fn cf_data_id<'a>(uc: &mut Unicorn<'a, ()>, from: u64) { uc.reg_write(RegisterX86::RAX, 2).unwrap(); }

fn cf_id<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let cf = emu_get_param(uc, 0).unwrap();
    let id = unsafe {
        GLOBAL_V.get(cf as usize - 1).unwrap().0
    };
    uc.reg_write(RegisterX86::RAX, id as u64).unwrap();
}

fn cf_len<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let cf = emu_get_param(uc, 0).unwrap();
    let len = unsafe {
        GLOBAL_V.get(cf as usize - 1).unwrap().1.len()
    };
    uc.reg_write(RegisterX86::RAX, len as u64).unwrap();
}

fn cf_maxlen<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let len = emu_get_param(uc, 0).unwrap();
    uc.reg_write(RegisterX86::RAX, len as u64).unwrap();
}

fn cf_getcstr<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let ptr = emu_get_param(uc, 0).unwrap();
    let s = cf_to_str(uc, ptr);
    println!("+ get cstr: {}", s);
    let ptr = emu_get_param(uc, 1).unwrap();
    uc.mem_write(ptr, s.as_bytes()).unwrap();
    uc.reg_write(RegisterX86::RAX, 1).unwrap();
}

fn cf_getbytes<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let ptr = emu_get_param(uc, 0).unwrap();
    let bytes = cf_to_bytes(uc, ptr);
    println!("+ get bytes: {}", hex::encode(&bytes));
    let start = emu_get_param(uc, 1).unwrap() as usize;
    let len = emu_get_param(uc, 2).unwrap() as usize;
    let ptr = emu_get_param(uc, 3).unwrap();
    println!("+ write to {:x}", ptr);
    uc.mem_write(ptr, &bytes[start..start+len]).unwrap();
    uc.reg_write(RegisterX86::RAX, 1).unwrap();
}

static mut GLOBAL_ETH: bool = false;

// io
fn eth_reset<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    unsafe { GLOBAL_ETH = false };
}

fn eth_next<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    if unsafe { GLOBAL_ETH } {
        uc.reg_write(RegisterX86::RAX, 0).unwrap();
    } else {
        unsafe { GLOBAL_ETH = true } ;
        uc.reg_write(RegisterX86::RAX, 1).unwrap();
    }
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
    let offset = unsafe {
        let old = GLOBAL_M;
        let new = GLOBAL_M + size;
        GLOBAL_M = new;
        old
    };
    println!("+ new: {:x}", MALLOC_BASE + offset);
    uc.reg_write(RegisterX86::RAX, MALLOC_BASE + offset).unwrap();
}

fn emulator_free<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let ptr: u64 = emu_get_param(uc, 0).unwrap();
    println!("+ free: {:x}", ptr);
}

fn emulator_memcpy<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let dst = emu_get_param(uc, 0).unwrap();
    let src = emu_get_param(uc, 1).unwrap();
    let len = emu_get_param(uc, 2).unwrap();
    let v = uc.mem_read_as_vec(src, len as usize).unwrap();
    uc.mem_write(dst, &v).unwrap();
    uc.reg_write(RegisterX86::RAX, src).unwrap();
}

fn emulator_memset<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    println!("FIXME: memset.");
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
    println!("FIXME: release.");
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
    let ptr = emu_get_param(uc, 1).unwrap();
    let s = emu_reads(uc, ptr).unwrap();
    let ptr = str_to_cf(uc, &s);
    uc.reg_write(RegisterX86::RAX, ptr).unwrap();
}

fn emulator_io_reg<'a>(uc: &mut Unicorn<'a, ()>, from: u64)
{
    let ptr = emu_get_param(uc, 1).unwrap();
    println!("+ io reg: {:?}", ptr);
    let s = cf_to_str(uc, ptr);
    println!("+ {}", s);
    
    let ret = match s.as_str()
    {
        "board-id" => vec_to_cf(uc, hex::decode("433234362d5755342d434600").unwrap()),
        "product-name" => vec_to_cf(uc, hex::decode("564d77617265372c3100").unwrap()),
        "boot-uuid" => vec_to_cf(uc, hex::decode("32453143463230452d414631332d344634432d414446312d34443431464142423836423500").unwrap()),
        "IOPlatformSerialNumber" => str_to_cf(uc, "VMxd6muhRqce"),
        "IOPlatformUUID" => str_to_cf(uc, "564D125C-23F9-7095-9EC9-983BCB8F2FD6"),
        "Gq3489ugfi" => vec_to_cf(uc, hex::decode("1548b8e035649e797e918931cab812aec7").unwrap()),
        "Fyp98tpgj" => vec_to_cf(uc, hex::decode("17bc9f170d6a2ae075299ae220ea43e2a7").unwrap()),
        "kbjfrfpoJU" => vec_to_cf(uc, hex::decode("18d9b7a86702cc6a8fab8f73c0ba2c2aed").unwrap()),
        "IOMACAddress" => vec_to_cf(uc, hex::decode("000c298f2fd6").unwrap()),
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:ROM" => vec_to_cf(uc, hex::decode("564d125c23f9").unwrap()),
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:MLB" => vec_to_cf(uc, hex::decode("634a5765795a67377934387631672e2e2e").unwrap()),
        "oycqAZloTNDm" => vec_to_cf(uc, hex::decode("55d0143a2eab41e70afa29de95b04cdffb").unwrap()),
        "abKPld1EcMni" => vec_to_cf(uc, hex::decode("e83ddc6931b80d867d0432225d8dc1a347").unwrap()),
        _ => 0 as _
    };
    uc.reg_write(RegisterX86::RAX, ret).unwrap();
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
        13 => { cf_id }
        14 => { cf_str_id }
        15 | 19 => { cf_len }
        16 => { cf_maxlen }
        17 => { cf_getcstr }
        18 => { cf_data_id }
        20 => { cf_getbytes }
        24 | 28 => { emulator_skip_0 }
        25 => { emulator_skip_0 } // TODO: _IOServiceGetMatchingServices
        26 => { eth_reset }
        27 => { eth_next }
        _ => { not_implement }
    };

    func(uc, _return_address);
}

pub fn call_0<'a>(uc: &mut Unicorn<'a, ()>, cert: &[u8]) -> Result<(u64, Vec<u8>)>
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

    let params = uc.mem_read_as_vec(PARAM_BASE, 24).unwrap();
    let ctx = u64::from_le_bytes(params[..8].try_into().unwrap());
    let ptr = u64::from_le_bytes(params[8..16].try_into().unwrap());
    let len = u64::from_le_bytes(params[16..24].try_into().unwrap());
    println!("+ return {:x}, {:x}, {:x}", ctx, ptr, len);

    Ok((ctx, uc.mem_read_as_vec(ptr, len as usize).unwrap()))
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
        _ => SYSCALL_MAX - 1,
    } as u32;

    SYSCALL_BASE + (SYSCALL_SIZE * col) as u64
}