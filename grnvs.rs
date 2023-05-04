// Provides Rust bindings of GRnvS's libraw
// 
// Copyright © 2023 Josef Schönberger
// 
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
// OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.



#![allow(non_camel_case_types, dead_code)] // Type names are given in C, hence `non_camel_case_types`.
                                           // We might not use all functions every time, hence `dead_code`.

use std::ffi::c_void;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug, Display, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};

use nix::errno::Errno;

// ----------------------- raw.h ----------------------

// Well, this is weird: "dylib" still links a static library. "static" produces a linking error:
// "static" links the library during compilation of each module, resulting in multiple definitions
// of the symbols in the library. "dylib" only links once when linking the executable, if the
// static library exists. Look, I didn't come up with these names! :-/
// For more information, see https://internals.rust-lang.org/t/meaning-of-link-kinds/2686
#[link(name = "raw", kind = "dylib")]
extern "C" {
    fn grnvs_open(ifname: *const i8, layer: i32) -> i32;
    fn grnvs_read(fd: i32, buf: *const c_void, maxlen: usize, timeout: *mut i32) -> isize;
    fn grnvs_write(fd: i32, buf: *const c_void, maxlen: usize) -> isize;
    fn grnvs_close(fd: i32) -> i32;
    fn grnvs_get_hwaddr(fd: i32) -> *const [u8; 6];
    fn grnvs_get_ipaddr(fd: i32) -> in_addr;
    fn grnvs_get_ip6addr(fd: i32) -> *const [u8; 16];
}

// Unfortunately, this is needed as a proxy struct since [u8; 4] returning directly is not FFI-safe.
#[repr(C, packed)]
struct in_addr {
    addr: [u8; 4],
}

#[repr(i32)]
pub enum Layer {
    SOCK_DGRAM = 2,
    SOCK_RAW = 3,
}

pub struct Socket(i32);

impl Socket {
    pub fn open(ifname: &str, layer: Layer) -> Self {
        unsafe {
            let c = CString::new(ifname).unwrap();
            Socket(grnvs_open(c.as_ptr(), layer as i32))
        }
    }

    /// Read the given amount of bytes from the Socket. You may optionally choose to provide a
    /// timeout argument (timeout in milliseconds).
    ///
    /// Returns the amount of bytes that were actually read.
    #[inline]
    pub fn read(&mut self, destination: &mut [u8], timeout: Option<&mut i32>) -> usize {
        unsafe {
            grnvs_read(
                self.0,
                destination.as_mut_ptr() as _,
                destination.len(),
                timeout
                    .map(|r| &mut *r as *mut i32)
                    .unwrap_or(std::ptr::null_mut()),
            ) as _
        }
    }

    /// Writes the given amount of bytes into the Socket.
    ///
    /// Returns the amount of bytes that were actually read if no error occured.
    #[inline]
    pub fn write(&mut self, source: &[u8]) -> Result<usize, Error> {
        let result = unsafe { grnvs_write(self.0, source.as_ptr() as _, source.len()) };
        if result < 0 {
            Err(Error(format!(
                "Error while writing to socket: {}",
                Errno::last()
            )))
        } else {
            Ok(result as _)
        }
    }

    #[inline]
    pub fn close(self) {} // let drop take care of this

    #[inline]
    pub fn get_hwaddr<'a>(&self) -> &'a [u8; 6] {
        unsafe { (grnvs_get_hwaddr(self.0)).as_ref().unwrap() }
    }

    #[inline]
    pub fn get_ipaddr(&self) -> Ipv4Addr {
        unsafe { grnvs_get_ipaddr(self.0).addr.into() }
    }

    #[inline]
    pub fn get_ip6addr(&self) -> Ipv6Addr {
        unsafe { (*grnvs_get_ip6addr(self.0)).into() }
    }
}

impl Drop for Socket {
    #[inline]
    fn drop(&mut self) {
        unsafe { grnvs_close(self.0) };
    }
}

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

// ---------------------------- checksum.h ------------------------------

#[link(name = "raw", kind = "dylib")]
extern "C" {
    fn icmp6_checksum(hdr: *const [u8; 40], payload: *const u8, len: usize) -> u16;
    fn get_crc32(frame: *const c_void, length: usize) -> u32;
}

#[inline]
pub fn icmp6_chksum(hdr: &[u8; 40], payload: &[u8]) -> u16 {
    unsafe { icmp6_checksum(hdr as _, payload.as_ptr(), payload.len()) }
}

#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    unsafe { get_crc32(data.as_ptr() as _, data.len()) }
}

// ----------------------------- hexdump.h -------------------------------

#[link(name = "raw", kind = "dylib")]
extern "C" {
    fn hexdump(buffer: *const c_void, len: isize);
    fn hexdump_str(buffer: *const c_void, len: isize) -> *const u8;
}

#[inline]
pub fn print_hexdump_to_stderr(data: &[u8]) {
    if data.len() > 17760 {
        eprintln!(
            "data buffer too long ({} bytes) for libraw's hexdump function",
            data.len()
        );
        std::process::exit(1);
    }
    unsafe { hexdump(data.as_ptr() as _, data.len() as _) };
}

#[inline]
pub fn hexdump_to_string(data: &[u8]) -> String {
    if data.len() > 17760 {
        eprintln!(
            "data buffer too long ({} bytes) for libraw's hexdump function",
            data.len()
        );
        std::process::exit(1);
    }
    let ptr = unsafe { hexdump_str(data.as_ptr() as _, data.len() as _) };
    let cstr = unsafe { CStr::from_ptr(ptr as _) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}
