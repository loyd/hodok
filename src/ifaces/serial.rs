use std::ffi::CString;
use std::io;

use libc::{open, write, read, close, O_RDWR, O_NOCTTY};
use libc::{c_void, c_uchar, c_int, c_uint, size_t};


#[repr(C)]
struct Termios {
    c_iflag: c_uint,
    c_oflag: c_uint,
    c_cflag: c_uint,
    c_lflag: c_uint,
    c_line: c_uchar,
    c_cc: [c_uchar; 32],
    c_ispeed: c_uint,
    c_ospeed: c_uint
}

const ECHO: c_uint = 0o0000010;
const ECHOE: c_uint = 0o0000020;
const ECHOK: c_uint = 0o0000040;
const ECHONL: c_uint = 0o0000100;
const ICANON: c_uint = 0o0000002;
const IEXTEN: c_uint = 0o0100000;
const ISIG: c_uint = 0o0000001;
const OCRNL: c_uint = 0o0000010;
const ONLCR: c_uint = 0o0000004;
const TCSANOW: c_int = 0;

extern {
    fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *mut Termios) -> c_int;
}


pub struct Serial(c_int);

impl Serial {
    pub fn open(device: &str) -> io::Result<Serial> {
        let c_str = CString::new(device).unwrap();
        let fd = unsafe { open(c_str.as_ptr(), O_RDWR|O_NOCTTY, 0) };
        check_io!(fd != -1);

        let mut options = Termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: 0,
            c_ospeed: 0
        };

        unsafe {
            check_io!(tcgetattr(fd, &mut options) == 0);
            options.c_lflag &= !(ECHO|ECHONL|ICANON|ISIG|IEXTEN);
            options.c_oflag &= !(ONLCR|OCRNL);
            check_io!(tcsetattr(fd, TCSANOW, &mut options) == 0);
        }

        Ok(Serial(fd))
    }

    #[inline]
    pub fn write(&self, buf: &[u8]) -> io::Result<()> {
        let bytes = unsafe {
            write(self.0, buf.as_ptr() as *const c_void, buf.len() as size_t)
        };

        check_io!(bytes as usize == buf.len());
        Ok(())
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> io::Result<()> {
        let bytes = unsafe {
            read(self.0, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t)
        };

        check_io!(bytes as usize == buf.len());
        Ok(())
    }
}

impl Drop for Serial {
    fn drop(&mut self) {
        unsafe { close(self.0); }
    }
}
