use spin::Mutex;
//use mopa::Any;

use core::ops::DerefMut;
use core::any::{Any, TypeId};
use core::sync::atomic::{AtomicUsize, Ordering};

use terminal::vga::FrameBuffer;
use terminal::Terminal;
use terminal::serial::SerialPort;

pub trait Module: Any + Send {
    fn receive_message(&mut self, op: u32, data: &[u8], out: &mut [u8]) -> u32;
}

pub type ModHandle = usize;
pub type AtomicHandle = AtomicUsize;

struct Handler {
    mods: &'static [&'static Mutex<Module>],
    current: AtomicHandle,
}

static HANDLER: Handler = Handler {
    mods: &[&MAIN, &FB, &COUNT, &TERMINAL, &SERIAL],
    current: AtomicHandle::new(MAIN_HANDLE),
};

static MAIN: Mutex<MainMod> = Mutex::new(MainMod);
static COUNT: Mutex<Counter> = Mutex::new(Counter(0));
static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(FB_HANDLE, SERIAL_HANDLE));
static FB: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort);

pub const MAIN_HANDLE: ModHandle = 0;
pub const COUNT_HANDLE: ModHandle = 2;
pub const TERMINAL_HANDLE: ModHandle = 3;
pub const FB_HANDLE: ModHandle = 1;
pub const SERIAL_HANDLE: ModHandle = 4;

impl Module {
    fn is<T: Module + Sized>(&self) -> bool {
        TypeId::of::<T>() == Any::get_type_id(self)
    }

    fn downcast_mut<T: Module + Sized>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            Some(unsafe { &mut *(self as *mut Module as *mut T) })
        } else {
            None
        }
    }

    fn downcast_ref<T: Module + Sized>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { &*(self as *const Module as *const T) })
        } else {
            None
        }
    }
}

pub fn send_message<T: Module, F>(reciever: ModHandle, op: u32, data: &[u8], callback: Option<F>)
where
    F: FnOnce(&mut T, &[u8], u32),
{
    let mut buffer = [0; 256];
    let (old_id, old, new) = {
        let old = HANDLER
            .current
            .swap(reciever, Ordering::SeqCst);
        (old, HANDLER.mods[old], HANDLER.mods[reciever])
    };

    let status = {
        new.lock().receive_message(op, data, &mut buffer)
    };
    
    HANDLER.current.store(old_id, Ordering::SeqCst);

    callback.map(|callback| {
        let mut m = old.lock();
        callback(
            m.downcast_mut()
                .expect("Callback expected wrong module type."),
            &mut buffer,
            status,
        )
    });
}

//pub fn respond

pub fn pause() {
    // TODO
}

struct Counter(u32);

impl Module for Counter {
    fn receive_message(&mut self, _: u32, _: &[u8], out: &mut [u8]) -> u32 {
        self.0 += 1;
        if out.len() < 4 {
            1
        } else {
            out[0] = (self.0 & 0xFF) as u8;
            out[1] = ((self.0 >> 8) & 0xFF) as u8;
            out[2] = ((self.0 >> 16) & 0xFF) as u8;
            out[3] = ((self.0 >> 24) & 0xFF) as u8;
            0
        }
    }
}

pub struct MainMod;

impl Module for MainMod {
    fn receive_message(&mut self, _: u32, _: &[u8], _: &mut [u8]) -> u32 {
        0
    }
}
