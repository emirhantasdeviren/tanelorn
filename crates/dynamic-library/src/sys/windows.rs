use windows_sys::Win32::Foundation::{FARPROC, HINSTANCE};
use windows_sys::Win32::System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryW};

use std::ffi::{CString, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

pub struct DynamicLibrary(HINSTANCE);

impl DynamicLibrary {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let os_path = <Path as AsRef<OsStr>>::as_ref(path);
        let wide_path = os_path
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        // SAFETY: `wide_path` is valid null-terminated UTF-16
        let module = unsafe { LoadLibraryW(wide_path.as_ptr()) };

        if module != 0 {
            // SAFETY: Function succeeded
            Some(Self(module))
        } else {
            None
        }
    }

    pub fn get<T: AsRef<str>>(&self, proc: T) -> FARPROC {
        let proc = CString::new(proc.as_ref()).unwrap();

        unsafe { GetProcAddress(self.0, proc.as_ptr().cast()) }
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        // SAFETY: Since self is alive and has valid HMODULE, we can safely free
        unsafe { FreeLibrary(self.0) };
    }
}
