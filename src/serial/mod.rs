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

    // int32_t serial_read_one(int64_t fd, uint8_t* buf)
    fn serial_read_one(fd: i64, buf: *mut u8) -> i32;

    // int32_t serial_write_one(int64_t fd, const uint8_t* byte)
    fn serial_write_one(fd: i64, byte: *const u8) -> i32;

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
        // Prepare the path
        let path = CString::new(path)?;

        // Open the serial device
        let fd = unsafe { serial_open(path.as_bytes_with_nul().as_ptr(), baudrate) };
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
        if fd < 0 {
            let errno = io::Error::last_os_error();
            return Err(errno);
        }
        Ok(Self { fd })
    }
}
impl Read for SerialDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for (pos, byte) in buf.iter_mut().enumerate() {
            // Read next byte
            let result = unsafe { serial_read_one(self.fd, byte) };
            if result < 0 {
                let errno = io::Error::last_os_error();
                return Err(errno);
            }

            // Fast return if newline
            if *byte == b'\n' {
                return Ok(pos + 1);
            }
        }
        Ok(buf.len())
    }
}
impl Write for SerialDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for byte in buf.iter() {
            // Write next byte
            let result = unsafe { serial_write_one(self.fd, byte) };
            if result < 0 {
                let errno = io::Error::last_os_error();
                return Err(errno);
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Drop for SerialDevice {
    fn drop(&mut self) {
        unsafe { serial_close(self.fd) }
    }
}
