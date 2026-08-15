use core::{ffi, ops::Deref};
use std::os::{fd::RawFd, unix::net::UnixStream};

use jni::{EnvUnowned, strings::JNIStr, sys::JNINativeMethod};
use libc::{dev_t, ino_t};

use crate::{error::ZygiskError, impl_sealing::Sealed};

pub use crate::raw::v4::transparent::*;

#[derive(Clone, Copy)]
pub struct V4;

impl Sealed for V4 {}

impl super::ZygiskApi<'_, V4> {
    #[inline(always)]
    pub fn with_companion<R>(
        &mut self,
        f: impl FnOnce(&mut UnixStream) -> R,
    ) -> Result<R, ZygiskError> {
        super::common::with_companion(
            || unsafe {
                let dispatch = self.dispatch();
                (dispatch.connect_companion_fn)(dispatch.base.this)
            },
            f,
        )
    }

    #[inline(always)]
    pub fn get_module_dir(&self) -> RawFd {
        unsafe {
            let dispatch = self.dispatch();
            (dispatch.get_module_dir_fn)(dispatch.base.this)
        }
    }

    #[inline(always)]
    pub fn set_option(&mut self, option: ZygiskOption) {
        unsafe {
            let dispatch = self.dispatch();
            (dispatch.set_option_fn)(dispatch.base.this, option)
        }
    }

    #[inline(always)]
    pub fn get_flags(&self) -> Result<StateFlags, ZygiskError> {
        let flags = unsafe {
            let dispatch = self.dispatch();
            (dispatch.get_flags_fn)(dispatch.base.this)
        };

        match StateFlags::from_bits(flags) {
            Some(flags) => Ok(flags),
            None => Err(ZygiskError::UnrecognizedStateFlag(flags)),
        }
    }

    /// # Safety
    ///
    #[inline(always)]
    pub unsafe fn hook_jni_native_methods(
        &mut self,
        env: EnvUnowned,
        class_name: impl Deref<Target = JNIStr>,
        mut methods: impl AsMut<[JNINativeMethod]>,
    ) {
        let class_name = class_name.deref();
        let methods = methods.as_mut();
        super::common::hook_jni_native_methods_with_len(
            env,
            class_name,
            methods,
            |env, name, ptr, len| unsafe {
                (self.dispatch().hook_jni_native_methods_fn)(env, name, ptr, len)
            },
        );
    }

    /// # Safety
    ///
    #[inline(always)]
    pub unsafe fn plt_hook_register<'a, 'b>(
        &'a mut self,
        device: dev_t,
        inode: ino_t,
        symbol: impl AsRef<ffi::CStr>,
        replacement: *const (),
        original: &'b mut *const (),
    ) where
        'b: 'a,
    {
        super::common::plt_hook_register_device(
            device,
            inode,
            symbol,
            replacement,
            original,
            |device, inode, symbol, replacement, original| unsafe {
                (self.dispatch().plt_hook_register_fn)(device, inode, symbol, replacement, original)
            },
        )
    }

    #[inline(always)]
    pub fn plt_hook_commit(&mut self) -> Result<(), ZygiskError> {
        super::common::plt_hook_commit(unsafe { (self.dispatch().plt_hook_commit_fn)() })
    }
}
