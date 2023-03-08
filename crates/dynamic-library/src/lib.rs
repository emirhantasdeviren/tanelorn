mod sys;

use crate::sys as lib_impl;

use std::path::Path;
use std::ptr::NonNull;

pub struct DynamicLibrary(lib_impl::DynamicLibrary);

impl DynamicLibrary {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        lib_impl::DynamicLibrary::new(path).map(Self)
    }

    pub fn get<T: AsRef<str>>(&self, proc: T) -> Option<NonNull<()>> {
        self.0
            .get(proc)
            .map(|p| unsafe { NonNull::new_unchecked(p as _) })
    }
}
