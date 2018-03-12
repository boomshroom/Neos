ENTRY(start)

SECTIONS {
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;
	kernel_start = .;

	/* First put the multiboot header, as it is required to be put very early
	   early in the image or the bootloader won't recognize the file format.
	   Next we'll put the .text section. */
	.text BLOCK(4K) : ALIGN(4K) {
		KEEP(*(.multiboot))
		*(.text)
		 *(.text.*)
	}

	/* Read-only data. */
	.rodata BLOCK(4K) : ALIGN(4K) {
		*(.rodata)
		*(.rodata.*)
	}

	/* Read-write data (initialized) */
	.data BLOCK(4K) : ALIGN(4K) {
		*(.data)
		*(.data.*)
	}

	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */

	kernel_end = .;
}
