pub mod vga;
pub mod serial;

use super::module::{self, Module, ModHandle};
use spin::Mutex;
use core::mem::drop;

#[repr(u32)]
pub enum Op {
	PutS,
	PutC,
	SetPos,
	SetColor,
	Clear,
}

pub const PUTS: u32 = Op::PutS as u32;
pub const PUTC: u32 = Op::PutC as u32;
pub const SETPOS: u32 = Op::SetPos as u32;
pub const SETCOLOR: u32 = Op::SetColor as u32;
pub const CLEAR: u32 = Op::Clear as u32;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

pub struct Terminal {
	vga: ModHandle,
	serial: ModHandle,
}

struct Store {
    output: [u8; 256],
    code: u32,
}

enum Progress {
	None,
	Partial(Store),
	Done(Store),
}

impl Terminal {
	pub const fn new(vga: ModHandle, serial: ModHandle) -> Terminal {
		Terminal{vga, serial}
	}
}

static STORE: Mutex<Progress> = Mutex::new(Progress::None);

impl Module for Terminal {
	fn receive_message(&mut self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
		module::send_message(self.vga, op, data, Some(|_, out, code| receive(out, code)));
		module::send_message(self.serial, op, data, Some(|_, out, code| receive(out, code)));

		loop {
			let s = STORE.lock();
			match *s {
				Progress::Done(ref s) => {
					output.copy_from_slice(&s.output);
					return s.code;
				}
				_ => {
					drop(s);
					module::pause();
				}
			}
		}
	}
}

enum Compare {
	Mismatch(usize),
	New,
	Old,
}

fn receive<'a>(data: &'a[u8], code: u32) {
	let mut store = STORE.lock();
	match *store {
		Progress::None => {
			let mut new = Store{output: [0; 256], code};
			new.output.copy_from_slice(data);
			*store = Progress::Partial(new);
		}
		Progress::Partial(ref s) => {
			let mut result = Compare::Old;
			for (i, (&old, &new)) in s.output.iter().zip(data.iter()).enumerate() {
				if old != new && old != 0 && new != 0 {
					result = Compare::Mismatch(i);
					break;
				} else if old == 0{
					result = Compare::New;
					break;
				} else if new == 0{
					result = Compare::Old;
					break;
				}
			}
			let mut output = [0; 256];
			output.copy_from_slice(match result {
				Compare::New => &data,
				Compare::Old => &s.output,
				Compare::Mismatch(i) => &data[..i],
			});
			let code = if s.code < code {
				s.code | (code << 16)
			} else {
				code | (s.code << 16)
			};
			*store = Progress::Done(Store{output, code});
		},
		Progress::Done(_) => unreachable!(),
	}
}