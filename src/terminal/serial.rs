use spin::Once;
use super::{CLEAR, PUTC, PUTS, SETCOLOR};
use super::super::module::Module;

pub struct SerialPort;

static INIT: Once<()> = Once::new();
const PORT: u16 = 0x3F8;

impl SerialPort {
    pub fn init() {
        INIT.call_once(|| {
            unsafe {
                outb(PORT + 1, 0x00);
                outb(PORT + 3, 0x80);
                outb(PORT + 0, 0x03);
                outb(PORT + 1, 0x00);
                outb(PORT + 3, 0x03);
                outb(PORT + 2, 0xC7);
                outb(PORT + 4, 0x0B);
            };
        });
    }

    fn putc(ch: u8) {
        let mut test = 0;
        unsafe {
            while test == 0 {
                test = inb(PORT + 5) & 0x20;
            }
            outb(PORT, ch);
        }
    }

    fn puts<S: AsRef<[u8]>>(s: S) {
        s.as_ref().iter().for_each(|&ch| SerialPort::putc(ch));
    }

    fn clear() {
        SerialPort::puts(b"\x1B[2K");
    }
}

impl Module for SerialPort {
    fn receive_message(&mut self, op: u32, data: &[u8], output: &mut [u8]) -> u32 {
        SerialPort::init();
        match op {
            PUTS => {
                SerialPort::puts(data);
                output
                    .iter_mut()
                    .zip(data)
                    .for_each(|(out, &input)| *out = input);
                0
            }
            /*SETPOS => {
                output.iter_mut().for_each(|b| *b = 0);
				if data.len() != 2 {
					return 1
				}
				if data[0] >= 80 || data[1] >= 25 {
					return 2
				}
				self.pos = (data[0], data[1]);
				0
			}*/
			/*SETCOLOR => {
                output.iter_mut().for_each(|b| *b = 0);
				if data.len() != 1 {
					return 1
				}
				self.color = ColorCode(data[0]);
                0
			}*/
            PUTC => {
                output.iter_mut().for_each(|b| *b = 0);
                if data.len() != 1 {
                    return 1;
                }
                output[0] = data[0];

                SerialPort::putc(data[0]);
                0
            }
            CLEAR => {
                output.iter_mut().for_each(|b| *b = 0);
                SerialPort::clear();
                0
            }
            _ => 3,
        }
    }
}

unsafe fn outb(port: u16, byte: u8) {
    asm!("outb %al, %dx" :: "{al}"(byte), "{dx}"(port));
}

unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port));
    result
}
