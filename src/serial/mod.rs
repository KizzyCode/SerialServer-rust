//! Provides OS-specific implementations

use crate::error::Error;
use std::{
    ffi::CString,
    io::{self, Read, Write},
};

extern "C" {
    // int64_t serial_open(const char* path, uint64_t bauds)
    fn serial_open(path: *const u8, bauds: u64) -> i64;

    // int64_t serial_duplicate(int64_t fd)
    fn serial_duplicate(fd: i64) -> i64;

    // int32_t serial_read_buf(uint8_t* buf, size_t* pos, size_t capacity, int64_t fd)
    fn serial_read_buf(buf: *mut u8, pos: *mut usize, capacity: usize, fd: i64) -> i32;

    // int32_t serial_write_buf(int64_t fd, const uint8_t* buf, size_t* pos, size_t capacity)
    fn serial_write_buf(fd: i64, buf: *const u8, pos: *mut usize, capacity: usize) -> i32;

    // int32_t serial_flush(int64_t fd)
    fn serial_flush(fd: i64) -> i32;

    // void serial_close(int64_t fd)
    fn serial_close(fd: i64);
}

/// A serial device
pub struct SerialDevice {
    /// The underlying file descriptor
    fd: i64,
}
impl SerialDevice {
    /// Opens a serial device
    pub fn new(path: &str, baudrate: u64) -> Result<Self, Error> {
        // Open the serial device
        let path = CString::new(path)?;
        let fd = unsafe { serial_open(path.as_bytes_with_nul().as_ptr(), baudrate) };

        // Validate the result
        if fd < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno.into());
        }
        Ok(Self { fd })
    }

    /// Tries to clone the serial device by duplicating the underlying file descriptor
    pub fn try_clone(&self) -> io::Result<Self> {
        // Duplicate file descriptor
        let fd = unsafe { serial_duplicate(self.fd) };

        // Validate the result
        if fd < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno);
        }
        Ok(Self { fd })
    }
}
impl Read for SerialDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Read into `buf`
        let mut pos = 0;
        let result = unsafe { serial_read_buf(buf.as_mut_ptr(), &mut pos, buf.len(), self.fd) };

        // Validate the result
        if result < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno);
        }
        Ok(pos)
    }
}
impl Write for SerialDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write from `buf`
        let mut pos = 0;
        let result = unsafe { serial_write_buf(self.fd, buf.as_ptr(), &mut pos, buf.len()) };

        // Validate the result
        if result < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno);
        }
        Ok(pos)
    }
    fn flush(&mut self) -> io::Result<()> {
        // Flush the device
        let result = unsafe { serial_flush(self.fd) };

        // Validate the result
        if result < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno);
        }
        Ok(())
    }
}
impl Drop for SerialDevice {
    fn drop(&mut self) {
        unsafe { serial_close(self.fd) }
    }
}
