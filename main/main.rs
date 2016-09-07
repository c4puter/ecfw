#![no_std]

extern "C" {
    fn mcu_init();
    fn board_init();
    fn do_toggle_led();
}

extern crate rust_support;

pub fn delay(t: u32)
{
    for _ in 0..t {
        rust_support::nop();
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

pub fn led_loop() {
    loop {
        for i in 0..5 {
            nblink(i, 80000);
            delay(800000);
        }
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    unsafe {
        mcu_init();
        board_init();
        do_toggle_led();
    }
    led_loop();
    return 0;
}
