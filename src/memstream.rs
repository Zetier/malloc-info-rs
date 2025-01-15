use libc::{c_char, FILE};
use std::ptr;
use thiserror::Error;

/// Custom error type for errors dealing with [`MemStream`]
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred when interfacing with libc
    #[error("libc error: {0}")]
    LibC(#[from] errno::Errno),

    /// An error occurred when managing the libc memstream buffer
    #[error("memstream invalid")]
    MemStreamInvalid,
}

/// A wrapper around a FILE pointer that writes to memory. We can also wrap this in a
/// [`std::io::Cursor`], giving us the ability to read from the underlying buffer.
#[derive(Debug)]
pub(crate) struct MemStream {
    pub(crate) fp: *mut FILE,
    buf: Box<*mut c_char>,
    buf_size: Box<usize>,
}

impl MemStream {
    /// Create a new [`MemStream`] using [`libc::open_memstream`]
    pub(crate) fn new() -> Result<Self, Error> {
        let mut buf = Box::new(ptr::null_mut::<c_char>());
        let mut buf_size = Box::new(0);

        // SAFETY: [`libc::open_memstream`] is dealing with raw pointers, so it's marked unsafe.
        // However the pointers we provide it are valid and heap allocated, and guaranteed to live
        // as long as the MemStream object.
        let fp = unsafe { libc::open_memstream(buf.as_mut(), buf_size.as_mut()) };

        if fp.is_null() {
            return Err(errno::errno().into());
        }

        // SAFETY: We can call this because we know that the buffer is valid and will be expanded
        // by libc if needed.
        let res = unsafe { libc::fflush(fp) };
        if res != 0 {
            // SAFETY: We know the file pointer is non-null, so this should be safe
            unsafe { libc::fclose(fp) };
            return Err(errno::errno().into());
        }

        if buf.is_null() {
            // SAFETY: We know the file pointer is non-null, so this should be safe
            unsafe { libc::fclose(fp) };
            return Err(Error::MemStreamInvalid);
        }

        Ok(Self { fp, buf, buf_size })
    }
}

impl AsRef<[u8]> for MemStream {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: The buffer is managed by [`libc::open_memstream`] and is guaranteed to be expanded
        // appropriately by libc
        unsafe { std::slice::from_raw_parts(*self.buf as _, *self.buf_size) }
    }
}

impl Drop for MemStream {
    fn drop(&mut self) {
        // SAFETY: We can call both of these functions because we are about to drop the MemStream
        // anyways.
        unsafe {
            libc::fclose(self.fp);
            libc::free(*self.buf as _);
        }
        self.fp = ptr::null_mut();
        *self.buf = ptr::null_mut();
        *self.buf_size = 0x0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hello_world() {
        let stream = MemStream::new().unwrap();
        let text = b"Hello, world!";
        unsafe {
            libc::fwrite(text.as_ptr() as _, 1, text.len(), stream.fp);
            libc::fflush(stream.fp);
        }
        assert_eq!(stream.as_ref(), b"Hello, world!");
    }

    #[test]
    fn no_flush() {
        let stream = MemStream::new().unwrap();
        let text = b"Hello, world!";
        unsafe {
            libc::fwrite(text.as_ptr() as _, 1, text.len(), stream.fp);
        }
        assert_eq!(stream.as_ref(), b"");
    }

    #[test]
    fn create() {
        let ms = MemStream::new().unwrap();
        assert!(!ms.fp.is_null());
        assert!(!ms.buf.is_null());
    }

    #[test]
    fn drop() {
        let ms = Box::leak(Box::new(MemStream::new().unwrap()));
        let pms = ms as *mut MemStream;
        unsafe {
            std::ptr::drop_in_place(pms);
        }
        assert!(ms.fp.is_null());
    }
}
