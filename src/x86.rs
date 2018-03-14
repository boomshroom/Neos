
#[repr(align(4096))]
pub struct Stack ([u8; 4096]);

#[no_mangle]
#[cfg(target_arch = "x86")]
pub static STACK: Stack = Stack([0; 4096]);

#[allow(unused_attributes)]
#[link_args = "-melf_i386 -nostdlib -static -Tx86.ld -gc-sections -n"]
extern {}

#[no_mangle]
#[naked]
pub unsafe extern fn _start() -> ! {
	//let top = STACK.last().unwrap();
	set_stack();

	asm!("call hello"); // Avoid prelude.

	loop {}
}

#[inline(always)]
#[naked]
unsafe fn set_stack() {
	asm!("lea esp, [STACK + 4096]" :::: "intel", "volatile");
}

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
