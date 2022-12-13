//! Provides OS-specific implementations

use crate::error::Error;
use std::{
    ffi::{c_char, CStr, CString},
    io,
};

extern "C" {
    // const char* serial_open(int64_t* fd, const uint8_t* path, uint64_t bauds)
    fn serial_open(fd: *mut i64, path: *const u8, bauds: u64) -> *const c_char;

    // const char* serial_duplicate(int64_t* fd, int64_t org)
    fn serial_duplicate(fd: *mut i64, org: i64) -> *const c_char;

    // const char* serial_read_buf(uint8_t* buf, size_t* pos, size_t capacity, int64_t fd)
    fn serial_read_buf(buf: *mut u8, pos: *mut usize, capacity: usize, fd: i64) -> *const c_char;

    // const char* serial_write_buf(int64_t fd, const uint8_t* buf, size_t* pos, size_t capacity)
    fn serial_write_buf(fd: i64, buf: *const u8, pos: *mut usize, capacity: usize) -> *const c_char;

    // const char* serial_flush(int64_t fd)
    fn serial_flush(fd: i64) -> *const c_char;

    // void serial_close(int64_t fd)
    fn serial_close(fd: i64);
}

/// Performs an FFI call
fn ffi<F>(f: F) -> Result<(), Error>
where
    F: FnOnce() -> *const c_char,
{
    // Call the function
    let result = f();

    // Validate the result
    if !result.is_null() {
        let error = unsafe { CStr::from_ptr(result) };
        let errno = io::Error::last_os_error();
        return Err(eio!("{} ({})", error.to_string_lossy(), errno));
    }
    Ok(())
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

        // Open the file
        let mut fd = -1;
        ffi(|| unsafe { serial_open(&mut fd, path.as_bytes_with_nul().as_ptr(), baudrate) })?;
        Ok(Self { fd })
    }

    /// Tries to clone the serial device by duplicating the underlying file descriptor
    pub fn try_clone(&self) -> Result<Self, Error> {
        // Duplicate file descriptor
        let mut fd = 1;
        ffi(|| unsafe { serial_duplicate(&mut fd, self.fd) })?;
        Ok(Self { fd })
    }

    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // Read into `buf`
        let mut pos = 0;
        ffi(|| unsafe { serial_read_buf(buf.as_mut_ptr(), &mut pos, buf.len(), self.fd) })?;
        Ok(pos)
    }

    /// Write a buffer into this writer, returning how many bytes were written
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        // Write from `buf`
        let mut pos = 0;
        ffi(|| unsafe { serial_write_buf(self.fd, buf.as_ptr(), &mut pos, buf.len()) })?;
        Ok(pos)
    }
    /// Attempts to write an entire buffer into this writer
    pub fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
        while !buf.is_empty() {
            match self.write(buf)? {
                0 => return Err(eio!("failed to write whole buffer")),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }
    /// Flush this output stream, ensuring that all intermediately buffered contents reach their destination
    pub fn flush(&mut self) -> Result<(), Error> {
        // Flush the device
        ffi(|| unsafe { serial_flush(self.fd) })
    }
}
impl Drop for SerialDevice {
    fn drop(&mut self) {
        unsafe { serial_close(self.fd) }
    }
}
