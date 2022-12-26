use std::ffi::{c_void, c_int, c_char, CStr, c_uint};
use crate::mac_serial::{MacSerial, self};
use libc::size_t;

unsafe extern "sysv64"
fn malloc(size: libc::size_t) -> *mut c_void
{
    let ptr = libc::malloc(size);
    log::info!("+ malloc: {:x}|{:x}", ptr as usize, size);
    ptr
}

unsafe extern "sysv64"
fn time(tm: *mut i64) -> i64
{
    libc::time(tm)
}

unsafe extern "sysv64"
fn free(ptr: *mut c_void)
{
    log::info!("+ free: {:x}", ptr as usize);
    libc::free(ptr)
}

unsafe extern "sysv64"
fn memcpy(
    dest: *mut c_void,
    src: *const c_void,
    len: libc::size_t,
    _destlen: libc::size_t,
) -> *mut c_void
{
    libc::memcpy(dest, src, len)
}

unsafe extern "sysv64"
fn memset(dest: *mut c_void, val: c_int, len: libc::size_t) -> *mut c_void
{
    libc::memset(dest, val, len)
}

unsafe extern "sysv64"
fn _stack_chk_fail()
{
    log::error!("+ stack check fail.");
}

#[repr(C)]
pub struct pthread_once_t {
    pub state: c_int,
    pub mutex: *mut c_void,
}

unsafe extern "sysv64"
fn pthread_once(
    _once_control: *mut pthread_once_t,
    init_routine: extern "sysv64" fn()
) -> c_int
{
    init_routine();
    0
}

unsafe extern "sysv64"
fn not_implement()
{
    log::error!("+ not implement.");
}

// for windows/linux
unsafe extern "sysv64"
fn arc4random() -> u32
{
    libc::rand() as u32
}

static STACK_CHK_GUARD: u64 = 0x1234567812345678;
static CFSTR_TAG: u64 = 0xabababababababab;
static PATCH_TAG: u64 = 0x2cacacac;
static PATCH_TAG2: u64 = 0xacacacac;
static KNULL: usize = 0x11111111;
static KTRUE: usize = 0x22222222;
static KFALSE:usize = 0x33333333;
static KALLOC:usize = 0x44444444;
static CF_TYPE_STRING:u32 = 1;
static CF_TYPE_DATA:u32 = 2;

#[repr(C)]
struct patch_data
{
    tag: *const u64,
    data: *const c_char,
}

impl patch_data
{
    pub extern "sysv64" fn new_str(s: *const c_char) -> *const patch_data
    {
        let ptr = patch_data {
            tag: &PATCH_TAG,
            data: s,
        };
        Box::leak(Box::new(ptr)) as *const patch_data
    }

    pub extern "sysv64" fn new_data(s: *const c_char) -> *const patch_data
    {
        let ptr = patch_data {
            tag: &PATCH_TAG2,
            data: s,
        };
        Box::leak(Box::new(ptr)) as *const patch_data
    }

    pub unsafe extern "sysv64" fn is_patch(d: *const patch_data) -> bool
    {
        *(*d).tag & 0x7fffffff == PATCH_TAG
    }

    pub unsafe extern "sysv64" fn get_from_cf(d: *const patch_data) -> *const c_char
    {
        // tag 为地址
        if *(*d).tag != CFSTR_TAG {
            log::error!("+ Got failed cf. {:x} | {:x}\n", *(*d).tag, d as usize);
            0 as _
        } else {
            *((d as *const u8).offset(16) as *const *const c_char)
        }
    }

    pub unsafe extern "sysv64" fn is_str(d: *const patch_data) -> bool
    {
        *(*d).tag != PATCH_TAG2
    }

    pub unsafe extern "sysv64" fn get_len(d: *const patch_data) -> usize
    {
        if Self::is_patch(d) {
            let len = libc::strlen((*d).data);
            if Self::is_str(d) {
                len
            } else {
                len / 2
            }
        }
        else {
            libc::strlen(Self::get_from_cf(d))
        }
    }

    pub unsafe extern "sysv64" fn get_ptr(d: *const patch_data) -> *const i8
    {
        if Self::is_patch(d) {
            unsafe { (*d).data as _ }
        }
        else {
            Self::get_from_cf(d) as _
        }
    }
}

// 字符串转换
#[repr(C)]
pub struct CFRange {
    pub location: isize,
    pub length: isize,
}
unsafe extern "sysv64" fn CFStringGetTypeID() -> u32 { CF_TYPE_STRING }
unsafe extern "sysv64" fn CFDataGetTypeID() -> u32 { CF_TYPE_DATA }
unsafe extern "sysv64"
fn CFGetTypeID(cf: *const patch_data) -> u32
{
    let ret =
    if patch_data::is_str(cf) {
        CF_TYPE_STRING
    } else {
        CF_TYPE_DATA
    };
    ret
}
unsafe extern "sysv64"
fn CFGetLength(cf: *const patch_data) -> u32
{
    let ret = patch_data::get_len(cf) as u32;
    ret
}
unsafe extern "sysv64"
fn CFStringGetMaximumSizeForEncoding(length: u32, _encoding: u32) -> u32
{
    length
}
unsafe extern "sysv64"
fn CFStringGetCString(
    cf: *const patch_data, 
    buffer: *mut c_char, 
    _buffer_size: i32, 
    _encoding: u32
) -> bool
{
    // fixup zero.
    let len = patch_data::get_len(cf);
    libc::memcpy(buffer as _, patch_data::get_ptr(cf) as _, len);
    *buffer.offset(len as isize) = 0;
    true
}
unsafe extern "sysv64"
fn CFDataGetBytes(
    cf: *const patch_data,
    range: CFRange, 
    buffer: *mut u8
)
{
    let s = CStr::from_ptr(patch_data::get_ptr(cf) as _).to_str().unwrap_unchecked();
    let d = hex::decode(s).unwrap_unchecked();
    libc::memcpy(buffer as _, (&d).as_ptr().offset(range.location) as _, range.length as usize);
}

// 特殊补丁
unsafe extern "sysv64"
fn sysctlbyname(name: *const c_char, oldp: *mut c_void, _oldlenp: *mut size_t, _newp: *mut c_void, _newlen: size_t) -> c_int
{
    let s = CStr::from_ptr(name).to_str().unwrap_unchecked();
    log::info!("sysctl: {}", s);

    match s
    {
        "kern.osversion" => {
            libc::strcpy(oldp as *mut i8, MacSerial::instance().osversion.as_ptr() as _);
        }
        "kern.osrevision" => {
            *(oldp as *mut u32) = MacSerial::instance().osrevision;
        }
        _ => {}
    }
    0
}

unsafe extern "sysv64" fn
IOServiceMatching(name: *const c_char) -> *const patch_data
{
    patch_data::new_str(name)
}

// 网卡补丁
unsafe extern "sysv64" fn
IOServiceGetMatchingServices(
    _masterPort: c_uint,
    _matching: *const patch_data,
    existing: *mut usize
) -> c_int
{
    // IOEthernetInterface
    let ptr = libc::malloc(8) as usize;
    *(ptr as *mut usize) = 1;
    *existing = ptr;
    0
}

static mut gg: u32 = 0;
unsafe extern "sysv64" fn
IOIteratorReset(ptr: *mut usize)
{
    // println!("+ call reset: {:x}", ptr as usize);
    // *(ptr as *mut usize) = 1;
    gg = 0;
}

unsafe extern "sysv64" fn
IOIteratorNext(ptr: *mut usize) -> usize
{
    println!("+ call next: {:x}", ptr as usize);
    // if *ptr == 1 {
    //     1
    // } else {
    //     *(ptr as *mut usize) = 2;
    //     0
    // }
    
    if gg == 0 {
        gg = gg + 1;
        1
    }
    else {
        0
    }
}

unsafe extern "sysv64" fn skip_0() -> usize { 0 }
unsafe extern "sysv64" fn skip_1() -> usize { 1 }

unsafe extern "sysv64" fn release(obj: *const c_void)
{
    log::warn!("+ TODO: release {:x}", obj as usize);
}

unsafe extern "sysv64" fn CFStringCreateWithCStringNoCopy(
    _alloc: *const c_void, 
    cStr: *const i8, 
    _encoding: u32
) -> *const patch_data
{
    patch_data::new_str(cStr)
}

unsafe extern "sysv64"
fn io_reg(name: *const c_char) -> *const patch_data
{
    let name = CStr::from_ptr(name).to_str().unwrap_unchecked();
    log::info!("+ io reg, {}", name);
    let ret = match name
    {
        "board-id" => patch_data::new_data(MacSerial::instance().board_id.as_ptr() as _),
        "product-name" => patch_data::new_data(MacSerial::instance().product_name.as_ptr() as _),
        "boot-uuid" => patch_data::new_data(MacSerial::instance().boot_uuid.as_ptr() as _),
        "IOPlatformSerialNumber" => patch_data::new_str(MacSerial::instance().serial_number.as_ptr() as _),
        "IOPlatformUUID" => patch_data::new_str(MacSerial::instance().uuid.as_ptr() as _),
        "Gq3489ugfi" => patch_data::new_data(MacSerial::instance().gq_serial.as_ptr() as _),
        "Fyp98tpgj" => patch_data::new_data(MacSerial::instance().fy_serial.as_ptr() as _),
        "kbjfrfpoJU" => patch_data::new_data(MacSerial::instance().kb_serial.as_ptr() as _),
        "IOMACAddress" => patch_data::new_data(MacSerial::instance().mac_address.as_ptr() as _),
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:ROM" => patch_data::new_data(MacSerial::instance().rom.as_ptr() as _),
        "4D1EDE05-38C7-4A6A-9CC6-4BCCA8B38C14:MLB" => patch_data::new_data(MacSerial::instance().mlb.as_ptr() as _),
        "oycqAZloTNDm" => patch_data::new_data(MacSerial::instance().oy_serial.as_ptr() as _),
        "abKPld1EcMni" => patch_data::new_data(MacSerial::instance().ab_serial.as_ptr() as _),
        _ => 0 as _
    };
    ret
}

unsafe extern "sysv64"
fn IORegistryEntryCreateCFProperty(
    _entry: c_uint,
    name: *const patch_data,
    _property: *const c_void
) -> *const patch_data
{
    let s = patch_data::get_ptr(name);
    io_reg(s)
}

pub fn register_fn(_dylib: &str, name: &str) -> usize
{
    match name
    {
        "_malloc" => malloc as usize,
        "_free" => free as usize,
        "____chkstk_darwin" => 0usize,
        "___stack_chk_guard" => &STACK_CHK_GUARD as *const u64 as usize,
        "_stack_chk_fail" => _stack_chk_fail as usize,
        "___memcpy_chk" => memcpy as usize,
        "___memset_chk" => memset as usize,
        "_time" => time as usize,
        // TODO: use other random
        // "_arc4random" => libc::rand as usize,
        "_pthread_once" => pthread_once as usize,
        "_arc4random" => arc4random as usize,
        // start to patch
        "_sysctlbyname" => sysctlbyname as usize,
        "_IOObjectRelease" => release as usize,
        "_IORegistryEntryFromPath" => skip_1 as usize,
        "_kIOMasterPortDefault" => &KNULL as *const usize as usize,
        "_kCFAllocatorDefault" => &KALLOC as *const usize as usize,
        "_kCFAllocatorNull" => &KNULL as *const usize as usize,
        "_CFStringCreateWithCStringNoCopy" => CFStringCreateWithCStringNoCopy as usize,
        "_IORegistryEntryCreateCFProperty" => IORegistryEntryCreateCFProperty as usize,
        "_CFGetTypeID" => CFGetTypeID as usize,
        "_CFStringGetTypeID" => CFStringGetTypeID as usize,
        "_CFStringGetLength" => CFGetLength as usize,
        "_CFStringGetMaximumSizeForEncoding" => CFStringGetMaximumSizeForEncoding as usize,
        "_CFStringGetCString" => CFStringGetCString as usize,
        "_CFDataGetTypeID" => CFDataGetTypeID as usize,
        "_CFDataGetLength" => CFGetLength as usize,
        "_CFDataGetBytes" => CFDataGetBytes as usize,
        "_CFRelease" => release as usize,
        "_IOServiceMatching" => IOServiceMatching as usize,
        // 网卡
        "_CFDictionaryCreateMutable" => skip_1 as usize,
        "_CFDictionarySetValue" => skip_0 as usize,
        "_IOServiceGetMatchingServices" => IOServiceGetMatchingServices as usize,
        "_IOIteratorReset" => IOIteratorReset as usize,
        "_IOIteratorNext" => IOIteratorNext as usize,
        "_IORegistryEntryGetParentEntry" => skip_0 as usize,
        "_CFStringGetSystemEncoding" => skip_1 as usize,
        "_kCFTypeDictionaryKeyCallBacks" => &KNULL as *const usize as usize,
        "_kCFTypeDictionaryValueCallBacks" => &KNULL as *const usize as usize,
        "_kCFBooleanTrue" => &KTRUE as *const usize as usize,
        "_kCFBooleanFalse" => &KFALSE as *const usize as usize,
        "___kCFBooleanFalse" => &KFALSE as *const usize as usize,
        "___kCFBooleanTrue" => &KTRUE as *const usize as usize,
        "___CFConstantStringClassReference" => &CFSTR_TAG as *const u64 as usize,
        // last
        _ => not_implement as usize,
    }
}