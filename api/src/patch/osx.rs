use std::ffi::{c_void, c_int, c_char, CStr, c_uint, CString};

use libc::size_t;

unsafe extern "sysv64"
fn malloc(size: libc::size_t) -> *mut c_void
{
    let ptr = libc::malloc(size);
    println!("+ malloc: {:x}|{:x}", ptr as usize, size);
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
    println!("+ free: {:x}", ptr as usize);
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
    println!("+ stack chk fail.");
}

static STACK_CHK_GUARD: u64 = 0x1234567812345678;

unsafe extern "sysv64"
fn not_implement()
{
    println!("+ not implement.");
}

// dynamic load function
fn get_dynamic(dylib: &str, name: &str) -> usize
{
    let dylib = CString::new(dylib).unwrap();
    let name = CString::new(name).unwrap();
    let func = unsafe {
        let handle = libc::dlopen(dylib.as_ptr(), libc::RTLD_NOW);
        libc::dlsym(handle, name.as_ptr()) as usize
    };
    if func == 0 {
        println!("+ not found func: {:?}, {:?}", dylib, name);
        not_implement as usize
    } else {
        func
    }
}

pub fn register_fn(dylib: &str, name: &str) -> usize
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
        "_pthread_once" => get_dynamic(dylib, "pthread_once"),
        "_arc4random" => libc::arc4random as usize,
        // forward
        "_sysctlbyname" | "_IOObjectRelease" | "_IORegistryEntryFromPath" |
        "_kIOMasterPortDefault" | "_kCFAllocatorDefault" | "_kCFAllocatorNull" |
        "_CFStringCreateWithCStringNoCopy" | "_IORegistryEntryCreateCFProperty" |
        "_CFGetTypeID" | "_CFStringGetTypeID" | "_CFStringGetLength" |
        "_CFStringGetMaximumSizeForEncoding" | "_CFStringGetCString" |
        "_CFDataGetTypeID" | "_CFDataGetLength" | "_CFDataGetBytes" |
        "_CFRelease" | "_IOServiceMatching" | "_CFDictionaryCreateMutable" |
        "_CFDictionarySetValue" | "_IOServiceGetMatchingServices" |
        "_IOIteratorReset" | "_IOIteratorNext" | "_IORegistryEntryGetParentEntry" |
        "_CFStringGetSystemEncoding" | "_kCFTypeDictionaryKeyCallBacks" | "_kCFTypeDictionaryValueCallBacks" |
        "_kCFBooleanTrue" | "_kCFBooleanFalse" |
        "___kCFBooleanFalse" | "___kCFBooleanTrue" |
        "___CFConstantStringClassReference" => {
            get_dynamic(dylib, &name[1..])
        }
        // last
        _ => not_implement as usize,
    }
}