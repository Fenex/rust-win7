use crate::ffi::OsString;
use crate::io::Result;
#[cfg(not(target_vendor = "win7"))]
use crate::mem::MaybeUninit;
use crate::os::windows::ffi::OsStringExt;
use crate::sys::pal::c;
use crate::sys::pal::winsock::{self, cvt};

/// ANSI code page. `gethostname` returns the name in this encoding.
#[cfg(target_vendor = "win7")]
const CP_ACP: u32 = 0;

pub fn hostname() -> Result<OsString> {
    winsock::startup();

    #[cfg(not(target_vendor = "win7"))]
    {
        // The documentation of GetHostNameW says that a buffer size of 256 is
        // always enough.
        let mut buffer = [const { MaybeUninit::<u16>::uninit() }; 256];
        // SAFETY: these parameters specify a valid, writable region of memory.
        cvt(unsafe { c::GetHostNameW(buffer.as_mut_ptr().cast(), buffer.len() as i32) })?;
        // Use `lstrlenW` here as it does not require the bytes after the nul
        // terminator to be initialized.
        // SAFETY: if `GetHostNameW` returns successfully, the name is nul-terminated.
        let len = unsafe { c::lstrlenW(buffer.as_ptr().cast()) };
        // SAFETY: the length of the name is `len`, hence `len` bytes have been
        //         initialized by `GetHostNameW`.
        let name = unsafe { buffer[..len as usize].assume_init_ref() };
        Ok(OsString::from_wide(name))
    }

    #[cfg(target_vendor = "win7")]
    {
        // `GetHostNameW` is Windows 8+. `gethostname` is in Windows 7's `ws2_32.dll`.
        // The `GetHostNameW` documentation says a buffer of 256 is always enough;
        // use the same size here.
        let mut buffer = [0u8; 256];
        // SAFETY: `buffer` is a valid, writable region of 256 bytes.
        cvt(unsafe { c::gethostname(buffer.as_mut_ptr(), buffer.len() as i32) })?;
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        if len == 0 {
            return Ok(OsString::new());
        }
        let mut wide = [0u16; 256];
        // SAFETY: the first `len` bytes of `buffer` are initialized, and `wide`
        // is a valid writable region of 256 UTF-16 code units.
        let n = unsafe {
            c::MultiByteToWideChar(
                CP_ACP,
                0,
                buffer.as_ptr(),
                len as i32,
                wide.as_mut_ptr(),
                wide.len() as i32,
            )
        };
        if n <= 0 {
            return Err(crate::io::Error::last_os_error());
        }
        Ok(OsString::from_wide(&wide[..n as usize]))
    }
}
