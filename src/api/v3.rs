use core::ffi;
use std::os::{fd::RawFd, unix::net::UnixStream};

use jni::{EnvUnowned, strings::JNIStr, sys::JNINativeMethod};

use crate::{error::ZygiskError, impl_sealing::Sealed};

pub use crate::raw::v3::transparent::*;

#[derive(Clone, Copy)]
pub struct V3;

impl Sealed for V3 {}

impl super::ZygiskApi<'_, V3> {
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

        super::common::parse_state_flags(flags)
    }

    /// # Safety
    ///
    #[inline(always)]
    pub unsafe fn hook_jni_native_methods<M: AsMut<[JNINativeMethod]>>(
        &mut self,
        env: EnvUnowned,
        class_name: &JNIStr,
        mut methods: M,
    ) {
        let methods = methods.as_mut();

        super::common::hook_jni_native_methods(
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
    pub unsafe fn plt_hook_register<'a, 'b, S>(
        &'a mut self,
        regex: S,
        symbol: S,
        new_func: *const (),
        old_func: &'b mut *const (),
    ) where
        'b: 'a,
        S: AsRef<ffi::CStr>,
    {
        super::common::plt_hook_register_basic(
            regex,
            symbol,
            new_func,
            old_func,
            |regex, symbol, new_func, old_func| unsafe {
                (self.dispatch().plt_hook_register_fn)(regex, symbol, new_func, old_func)
            },
        )
    }

    /// # Safety
    ///
    #[inline(always)]
    pub unsafe fn plt_hook_exclude<S>(&mut self, regex: S, symbol: S)
    where
        S: AsRef<ffi::CStr>,
    {
        super::common::plt_hook_exclude(regex, symbol, |regex, symbol| unsafe {
            (self.dispatch().plt_hook_exclude_fn)(regex, symbol)
        })
    }

    #[inline(always)]
    pub fn plt_hook_commit(&mut self) -> Result<(), ZygiskError> {
        super::common::plt_hook_commit(unsafe { (self.dispatch().plt_hook_commit_fn)() })
    }
}
