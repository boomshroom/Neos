#![allow(mutable_transmutes)]

use core::mem::transmute;

pub trait Module {
	fn receive_message(&mut self, op: u32, data: &[u8], out: &mut [u8]) -> u32;
}

type ModHandle = usize;

struct Handler {
	mods: &'static mut[&'static Module],
    current: ModHandle,
}

static mut HANDLER : Handler = Handler{ mods: &mut [&MAIN, &FB, &COUNT], current: 0 };

static MAIN : MainMod = MainMod;
static FB : FrameBuffer = FrameBuffer { fb: 0xb8000 as *mut [(u8,u8); 80 * 25], pos: 0, color: 0x07 };
static COUNT : Counter = Counter(0);

pub fn send_message(reciever: ModHandle, op: u32, data: &[u8], callback: fn(&mut Module, &[u8], u32)) {
    let mut buffer = [0; 256];
    unsafe {
    	let rec : &'static mut Module = transmute(HANDLER.mods[reciever]);
    	let status = rec.receive_message(op, data, &mut buffer);
    	let cur : &'static mut Module = transmute(HANDLER.mods[HANDLER.current]);
    	callback(cur, &mut buffer, status);
	}
}

struct FrameBuffer {
    fb: *mut [(u8, u8); 80 * 25],
    pos: u16,
    color: u8,
}

unsafe impl Sync for FrameBuffer {}
unsafe impl Sync for Handler {}

impl Module for FrameBuffer {
	fn receive_message(&mut self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
		match op {
			0 => {
				let &mut FrameBuffer{ref mut fb, ref mut pos, ref color} = self;
				let writen = (unsafe{&mut **fb})[*pos as usize ..].iter_mut().zip(data).map(|(out, &ch)| *out = (ch, *color)).count();
				output.copy_from_slice(&data[..writen]);
				*pos += writen as u16;
                if writen == data.len() { 0 } else { 1 }
			}
			1 => {
                output.iter_mut().for_each(|b| *b = 0);
				if data.len() != 2 {
					return 1
				}
				if data[0] >= 80 || data[1] >= 25 {
					return 2
				}
				self.pos = data[0] as u16 + (data[1] as u16 * 80);
				0
			}
			2 => {
                output.iter_mut().for_each(|b| *b = 0);
				if data.len() != 1 {
					return 1
				}
				self.color = data[0];
                return 0
			}
			_ => 3,
		}
	}
}

struct Counter (usize);

impl Module for Counter {
    fn receive_message(&mut self, _: u32, _: &[u8], out: &mut[u8]) -> u32 {
        self.0 += 1;
        if out.len() < 4 {
            1
        } else {
        	let ptr : &'static mut usize = unsafe { ::core::mem::transmute(&out[0]) };
            *ptr = self.0;
            0
        }
    }
}

struct MainMod;

impl Module for MainMod {
	fn receive_message(&mut self, _: u32, _: &[u8], _: &mut[u8]) -> u32 {
		0
	}
}