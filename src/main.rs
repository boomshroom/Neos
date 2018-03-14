#![feature(lang_items,start,link_args,naked_functions,asm,attr_literals,global_asm,const_fn,specialization,nll)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use] extern crate lazy_static;

mod module;
mod terminal;

use terminal::Op;

#[cfg(target_arch = "x86")]
mod x86;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub fn _start() -> !{
	hello();
	loop {}
}

#[no_mangle]
pub extern "C" fn hello() {
	module::send_message(1, Op::Clear as u32, &[], Some(|_, _, _| {
		module::send_message(3, Op::PutS as u32, b"Hello, World!", Some(|_, _, _| {
			module::send_message(3, Op::PutS as u32, b"\nTerminal in working order!", Some(|_, _, _| loop {}));
		}));
	}));
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments, _file: &'static str, _line: u32, _column: u32) -> ! {
	loop{}
}