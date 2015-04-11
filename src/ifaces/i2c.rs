use std::ffi::CString;
use std::io::Error as IoError;
use std::io::Result as IoResult;
use std::mem;

use libc::{open, write, read, close, O_RDWR, c_int, size_t};


const I2C_SLAVE: c_int = 0x0703;


extern {
    fn ioctl(fd: c_int, req: c_int, ...) -> c_int;
}


macro_rules! check_io(
    ($cond:expr) =>
        (if !$cond { return Err(IoError::last_os_error()); })
);

pub struct I2C {
    fd: c_int,
    addr: u16
}

impl I2C {
    pub fn open(bus: u32, addr: u16) -> IoResult<I2C> {
        let c_str = CString::new(format!("/dev/i2c-{}", bus)).unwrap();
        let fd = unsafe { open(c_str.as_ptr(), O_RDWR, 0) };

        check_io!(fd != 0);
        check_io!(unsafe { ioctl(fd, I2C_SLAVE, addr as c_int) >= 0 });

        Ok(I2C { fd: fd, addr: addr })
    }

    pub fn write(&self, buf: &[u8]) -> IoResult<()> {
        let bytes = unsafe {
            write(self.fd, mem::transmute(buf.as_ptr()), buf.len() as size_t)
        };

        check_io!(bytes as usize == buf.len());
        Ok(())
    }

    pub fn read(&self, reg: u8, buf: &mut [u8]) -> IoResult<()> {
        check_io!(unsafe { write(self.fd, mem::transmute(&reg), 1) == 1 });

        let bytes = unsafe {
            read(self.fd, mem::transmute(buf.as_mut_ptr()), buf.len() as size_t)
        };

        check_io!(bytes as usize == buf.len());
        Ok(())
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        unsafe { close(self.fd); }
    }
}