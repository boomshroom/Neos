#![feature(lang_items,start,link_args,naked_functions,asm,attr_literals,global_asm,const_fn,specialization)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate volatile;
extern crate spin;

#[macro_use] extern crate lazy_static;

mod module;
mod terminal;

use terminal::vga::Op;

#[repr(align(4096))]
#[cfg(target_arch = "x86")]
pub struct Stack ([u8; 4096]);

#[no_mangle]
#[cfg(target_arch = "x86")]
pub static STACK: Stack = Stack([0; 4096]);

#[no_mangle]
#[naked]
#[cfg(target_arch = "x86")]
pub unsafe extern fn start() -> ! {
	//let top = STACK.last().unwrap();
	set_stack();

	asm!("call hello"); // Avoid prelude.

	loop {}
}

#[inline(always)]
#[naked]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn set_stack() {
	#[cfg(target_arch = "x86")] {
		asm!("lea esp, [STACK + 4096]" :::: "intel", "volatile");
	}
	#[cfg(target_arch = "x86_64")] {
		asm!("lea rsp, [STACK + 4096]" :::: "intel", "volatile");
	}
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub fn _start() -> !{
	hello();
	loop {}
}

#[no_mangle]
pub extern "C" fn hello() {
	module::send_message(1, Op::Clear as u32, &[], Some(|_, _, _| {
		module::send_message(1, Op::PutS as u32, b"Hello, World!", Some(|_, _, _| loop {}));
	}));
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments, _file: &'static str, _line: u32, _column: u32) -> ! {
	loop{}
}


#[cfg(target_arch = "x86")] 
pub mod mb_header {
	global_asm!(r#"
		.pushsection .multiboot
	    multiboot:

	    .set MAGIC, 0x1BADB002
	    .set FLAGS, 1<<0 | 1<<1
	    .set CHECKSUM, -(MAGIC + FLAGS)

	    .align 4

	    .long MAGIC
	    .long FLAGS
	    .long CHECKSUM
	    .popsection
	"#);

	/*
	#[repr(C)]
	pub struct MBHeader {
		magic: i32,
		flags: i32,
		check: i32,
	}

	const MAGIC : i32 = 0x1BADB002;
	const FLAGS : i32 = 1<<0 | 1<<1;
	const CHECKSUM : i32 = -(MAGIC + FLAGS);
	

	global_asm!(r#".pushsection .multiboot"#);
	#[no_mangle] pub static MULTIBOOT : MBHeader = MBHeader {magic: MAGIC, flags: FLAGS, check: CHECKSUM};
	global_asm!(r#".popsection"#);
	*/
}
