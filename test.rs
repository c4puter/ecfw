#![no_std]
#![crate_type="staticlib"]
#![feature(lang_items)]

#[lang="eh_personality"] extern fn eh_personality() {}

#[lang="panic_fmt"]
pub fn panic_fmt(_fmt: &core::fmt::Arguments, _file_line: &(&'static str, usize)) -> !{
    loop { }
}

#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> () {
    loop { }
}

extern "C" {
    fn do_toggle_led() -> ();
    fn do_nothing() -> ();
}

pub fn do_thing() -> () {
    loop {
        unsafe{ do_toggle_led(); }

        let mut x: i32 = 0;
        while x < 400000 {
            x += 1;
            unsafe{ do_nothing(); }
        }
    }
}

#[no_mangle]
pub extern "C" fn do_thing_c() {
    do_thing();
}
