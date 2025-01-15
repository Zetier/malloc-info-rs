//! This crate provides safe access to glibc's `malloc_info` function. See the
//! [malloc_info(3)](https://man7.org/linux/man-pages/man3/malloc_info.3.html) page for details on
//! that function.
//!
//! # Example
//! ```rust
//! # use malloc_info::malloc_info;
//! let info = malloc_info().expect("malloc_info");
//! println!("{:#?}", info);
//! ```
//!
//! # Caveats
//! `malloc_info` is a glibc-specific function and is not available on all platforms. This crate
//! will not work on platforms where `malloc_info` is not available.
//!
//! `malloc_info` will only report heap statistics for the glibc heap. If your program uses a
//! different heap implementation, for example by `#[global_allocator]` or by using a different
//! libc, `malloc_info` will not report statistics for that heap.

use errno::Errno;
use thiserror::Error;

pub mod info;
mod memstream;

use memstream::MemStream;

/// Internal representation for errors occurring during the [`malloc_info`] call. This is private so
/// we can modify it without breaking the public API.
#[derive(Debug, Error)]
enum ErrorRepr {
    /// An error occurred when interfacing with libc
    #[error("libc error: {0}")]
    LibC(#[from] Errno),

    /// An internal error occurred when interfacing with the memstream module
    #[error(transparent)]
    Memstream(#[from] memstream::Error),

    /// An error occurred when parsing the XML output of `malloc_info`
    #[error("failed to parse malloc_info XML output: {0}")]
    Xml(#[from] quick_xml::DeError),
}

/// Custom error type for errors occurring during the [`malloc_info`] call
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(#[from] ErrorRepr);

/// Safely get information from [`libc::malloc_info`]. See library-level documentation for more
/// information.
pub fn malloc_info() -> Result<info::Malloc, Error> {
    fn malloc_info() -> Result<info::Malloc, ErrorRepr> {
        let mem_stream = MemStream::new()?;
        let mut cursor = std::io::Cursor::new(mem_stream);

        // SAFETY: `libc::malloc_info` is marked unsafe because it is in the libc crate and it deals
        // with raw pointers. Being in the libc crate is not inherently unsafe. The raw pointer it
        // deals with is a pointer to a FILE struct, taken from the mem_stream object, which we control
        // and have exclusive, mutable access to in this function, ensuring no other code can access
        // it.
        //
        // The same logic applies to `libc::fflush`.
        unsafe {
            if libc::malloc_info(0, cursor.get_mut().fp) != 0 {
                return Err(errno::errno().into());
            }

            if libc::fflush(cursor.get_mut().fp) != 0 {
                return Err(errno::errno().into());
            }
        }

        Ok(quick_xml::de::from_reader(&mut cursor)?)
    }
    malloc_info().map_err(Error::from)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn call_from_async() {
        let _ = tokio::task::spawn(async { malloc_info().expect("malloc_info") }).await;
    }
}
