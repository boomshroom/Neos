#![feature(lang_items,start,link_args,naked_functions,asm,attr_literals,global_asm)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate volatile;

mod module;

use volatile::WriteOnly;

#[allow(unused_attributes)]
#[link_args = "-nostdlib -static -estart -Tx86.ld -gc-sections -n"]
extern {}

#[repr(align(4096))]
pub struct Stack ([u8; 4096]);

#[no_mangle] pub static STACK: Stack = Stack([0; 4096]);

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
pub extern "C" fn hello() {
	let fb = unsafe { &mut *(0xb8000 as *mut [[WriteOnly<(u8, u8)>;80];25]) };
	fb.iter_mut().flat_map(|x|x.iter_mut()).for_each(|cell| cell.write((' ' as u8, 0x00)));
    
    module::send_message(1, 0, b"Hello, World!", |_, _, _| loop {});
}

#[lang = "eh_personality"] #[no_mangle] pub extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}

#[no_mangle] pub extern "C" fn _Unwind_Resume() -> ! { loop {} }

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
