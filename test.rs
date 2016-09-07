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

fn delay(t: u32)
{
    let mut x: u32 = 0;
    while x < t {
        x += 1;
        unsafe{ do_nothing(); }
    }
}

fn blink(t: u32) {
    unsafe{ do_toggle_led(); }
    delay(t);
    unsafe{ do_toggle_led(); }
    delay(t);
}

fn nblink(times: u32, t: u32) {
    for _ in 0..times {
        blink(t);
    }
}

pub fn do_thing() -> () {
    loop {
        for i in 0..5 {
            nblink(i, 40000);
            delay(400000);
        }
    }
}

#[no_mangle]
pub extern "C" fn do_thing_c() {
    do_thing();
}
