#![feature(lang_items, start, link_args, naked_functions, asm, attr_literals, global_asm,
           const_fn, specialization, nll)]
#![feature(get_type_id)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate spin;
extern crate volatile;

// #[macro_use] extern crate lazy_static;

mod module;
mod terminal;

use terminal::Op;
use module::{MainMod, FB_HANDLE, TERMINAL_HANDLE};

#[cfg(target_arch = "x86")]
mod x86;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub fn _start() -> ! {
    hello();
    loop {}
}

#[no_mangle]
pub extern "C" fn hello() {
    module::send_message(
        FB_HANDLE,
        Op::Clear as u32,
        &[],
        Some(|_: &mut MainMod, _: &[u8], _| {
            module::send_message(
                FB_HANDLE,
                Op::PutS as u32,
                b"Hello, World!",
                Some(|_: &mut MainMod, _: &[u8], _| {
                    module::send_message::<MainMod, fn(&mut MainMod, &[u8], u32)>(
                        FB_HANDLE,
                        Op::PutS as u32,
                        b"\nTerminal in working order!",
                        None,
                    );
                }),
            );
        }),
    );
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    _msg: core::fmt::Arguments,
    _file: &'static str,
    _line: u32,
    _column: u32,
) -> ! {
    loop {}
}
