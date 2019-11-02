extern crate ws;
extern crate url;

use std::thread;
use std::sync::mpsc::channel;
//use libc::c_char;
use std::os::raw::c_char;
use std::ffi::CStr;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PollWSStatus {
    CLOSED,
    OPENING,
    OPEN,
    ERROR,
}

// working version?

enum SocketMessage {
    Connect,
    Disconnect,
    Message(ws::Message),
}


pub struct PollWSSocket {
    status: PollWSStatus,
    sender: ws::Sender,
    thread: Option<thread::JoinHandle<()>>,
    rx: std::sync::mpsc::Receiver<SocketMessage>,
    message: Option<ws::Message>,
}

impl PollWSSocket {
    fn poll(&mut self) -> bool {
        let mut got_message = false;
        while let Ok(msg) = self.rx.try_recv() {
            println!("We got a message!");
            match msg {
                SocketMessage::Connect => {
                    self.status = PollWSStatus::OPEN;
                },
                SocketMessage::Disconnect => {
                    self.status = PollWSStatus::CLOSED;
                    break
                },
                SocketMessage::Message(msg) => {
                    self.message = Some(msg);
                    got_message = true;
                    break
                },
            };
        };
        got_message
    }

    fn send(&mut self, msg: ws::Message) {
        match self.status {
            PollWSStatus::OPEN => self.sender.send(msg).unwrap_or_default(),
            _ => (),
        };
    }

    fn close(&mut self) {
        match self.status {
            PollWSStatus::OPEN | PollWSStatus::OPENING => {
                self.sender.shutdown().unwrap_or_default();
                self.status = PollWSStatus::CLOSED;
            },
            _ => (),
        }
        if let Some(handle) = self.thread.take() {
            handle.join().unwrap_or_default();
        };
    }

    fn new(url: String) -> PollWSSocket {
        println!("Trying to make a socket!");
        let (tx, rx) = channel();
        let mut socket = ws::Builder::new().build(move |_| {
            tx.send(SocketMessage::Connect).unwrap_or_default();
            println!("Sending the message that we connecte!");
            let tx2 = tx.clone();
            move |msg| {
                tx2.send(SocketMessage::Message(msg)).unwrap_or_default();
                Ok(())
            }
        }).unwrap();
        let handle = socket.broadcaster();
        let t = thread::spawn(move || {
            let actual_url = url::Url::parse(&url).unwrap();
            match socket.connect(actual_url) {
                Ok(_) => {
                    println!("So far so good!");
                    socket.run();
                    true
                },
                Err(err) => {
                    println!("Failed to create WebSocket due to: {:?}", err);
                    false
                },
            };
        });

        PollWSSocket {
            status: PollWSStatus::OPENING,
            sender: handle,
            thread: Some(t),
            rx: rx,
            message: None,
        }
    }
}

#[no_mangle]
pub extern fn pollws_open(url: *const c_char) -> *mut PollWSSocket {
    let url = unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() };
    Box::into_raw(Box::new(PollWSSocket::new(url)))
}

#[no_mangle]
pub extern fn pollws_close(ctx: *mut PollWSSocket) {
    let ctx = unsafe{&mut *ctx};
    ctx.close();

    // take ownership and drop
    let b = unsafe{ Box::from_raw(ctx) };
    drop(b);
}

#[no_mangle]
pub extern fn pollws_status(ctx: *mut PollWSSocket) -> PollWSStatus {
    let ctx = unsafe{&*ctx};
    ctx.status
}

// #[no_mangle]
// pub extern fn pollws_send(ctx: *mut PollWSSocket, msg: *const u8, msg_len: u32) {
//     let ctx = unsafe{&mut *ctx};
//     let bytes = unsafe { std::slice::from_raw_parts(msg, msg_len) };
//     ctx.send(ws_msg);
// }

#[no_mangle]
pub extern fn pollws_send(ctx: *mut PollWSSocket, msg: *const c_char) {
    let ctx = unsafe{&mut *ctx};
    let msg = unsafe { CStr::from_ptr(msg).to_string_lossy().into_owned() };
    ctx.send(ws::Message::text(msg));
}

#[no_mangle]
pub extern fn pollws_poll(ctx: *mut PollWSSocket) -> bool {
    let ctx = unsafe{&mut *ctx};
    ctx.poll()
}

#[no_mangle]
pub extern fn pollws_get(ctx: *mut PollWSSocket, dest: *mut u8, dest_size: u32) -> u32 {
    let ctx = unsafe{&mut *ctx};
    match ctx.message.take() {
        Some(msg) => {
            let ncopy = msg.len();
            if ncopy < (dest_size as usize) {
                unsafe {
                    std::ptr::copy_nonoverlapping(msg.into_data().as_ptr(), dest, ncopy);
                }
                ncopy as u32
            } else {
                0
            }
        },
        None => 0,
    }
}

#[no_mangle]
pub extern fn pollws_pop(ctx: *mut PollWSSocket, dest: *mut u8, dest_size: u32) -> u32 {
    if pollws_poll(ctx) {
        pollws_get(ctx, dest, dest_size)
    } else {
        0
    }
}