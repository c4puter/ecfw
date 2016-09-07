#![no_std]
#![crate_type="staticlib"]
#![feature(lang_items)]
#![feature(asm)]

#[lang="eh_personality"] extern fn eh_personality() {}

#[lang="panic_fmt"]
pub fn panic_fmt(_fmt: &core::fmt::Arguments, _file_line: &(&'static str, usize)) -> !{
    loop { }
}

#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> () {
    loop { }
}

pub fn nop()
{
    unsafe{asm!("nop" : : : : "volatile");}
}

