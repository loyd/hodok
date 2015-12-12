use std::ffi::CString;
use std::io;
use std::mem;

use libc::{open, write, read, close, O_RDWR, c_void, c_int, size_t};


const I2C_SLAVE: c_int = 0x0703;

extern {
    fn ioctl(fd: c_int, req: c_int, ...) -> c_int;
}


macro_rules! check_io(
    ($cond:expr) =>
        (if !$cond { return Err(io::Error::last_os_error()); })
);

pub struct I2C(c_int);

impl I2C {
    pub fn open(bus: &str, addr: u16) -> io::Result<I2C> {
        let c_str = CString::new(bus).unwrap();
        let fd = unsafe { open(c_str.as_ptr(), O_RDWR, 0) };

        check_io!(fd != -1);
        check_io!(unsafe { ioctl(fd, I2C_SLAVE, addr as c_int) >= 0 });

        Ok(I2C(fd))
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
    pub fn read(&self, reg: u8, buf: &mut [u8]) -> io::Result<()> {
        check_io!(unsafe { write(self.0, mem::transmute(&reg), 1) == 1 });

        let bytes = unsafe {
            read(self.0, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t)
        };

        check_io!(bytes as usize == buf.len());
        Ok(())
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        unsafe { close(self.0); }
    }
}
