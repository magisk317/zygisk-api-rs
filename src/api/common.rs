use core::{ffi, mem, ptr::NonNull};
use std::os::{fd::FromRawFd, unix::net::UnixStream};

use jni::{EnvUnowned, strings::JNIStr, sys::JNINativeMethod};
use libc::{c_char, c_int};

use crate::{error::ZygiskError, utils};

pub(crate) fn with_companion<R>(
    connect: impl FnOnce() -> c_int,
    f: impl FnOnce(&mut UnixStream) -> R,
) -> Result<R, ZygiskError> {
    match connect() {
        -1 => Err(ZygiskError::ConnectCompanionError),
        fd => {
            // SAFETY: Zygisk returns an owned connected descriptor on success.
            let mut companion_sock = unsafe { UnixStream::from_raw_fd(fd) };
            Ok(f(&mut companion_sock))
        }
    }
}

pub(crate) fn hook_jni_native_methods(
    env: EnvUnowned,
    class_name: &JNIStr,
    methods: &mut [JNINativeMethod],
    invoke: impl FnOnce(EnvUnowned, *const c_char, NonNull<JNINativeMethod>, c_int),
) {
    let ptr = NonNull::new(methods.as_mut_ptr()).unwrap_or_else(NonNull::dangling);
    invoke(env, class_name.as_ptr(), ptr, methods.len() as c_int);
}

pub(crate) fn plt_hook_register_basic<'b, S>(
    regex: S,
    symbol: S,
    new_func: *const (),
    old_func: &'b mut *const (),
    invoke: impl FnOnce(*const c_char, *const c_char, *const libc::c_void, &'b mut *const libc::c_void),
) where
    S: AsRef<ffi::CStr>,
{
    let regex = regex.as_ref();
    let symbol = symbol.as_ref();

    let _: () = utils::ShapeAssertion::<*const (), extern "C" fn()>::ASSERT;
    // SAFETY: ShapeAssertion proves the two pointer representations match.
    let old_func =
        unsafe { mem::transmute::<&'b mut *const (), &'b mut *const libc::c_void>(old_func) };
    invoke(
        regex.to_bytes_with_nul().as_ptr().cast(),
        symbol.to_bytes_with_nul().as_ptr().cast(),
        new_func.cast(),
        old_func,
    );
}

pub(crate) fn plt_hook_register_device<'b>(
    device: libc::dev_t,
    inode: libc::ino_t,
    symbol: impl AsRef<ffi::CStr>,
    replacement: *const (),
    original: &'b mut *const (),
    invoke: impl FnOnce(
        libc::dev_t,
        libc::ino_t,
        *const c_char,
        *const libc::c_void,
        &'b mut *const libc::c_void,
    ),
) {
    let symbol = symbol.as_ref();
    let _: () = utils::ShapeAssertion::<*const (), extern "C" fn()>::ASSERT;
    // SAFETY: ShapeAssertion proves the two pointer representations match.
    let original =
        unsafe { mem::transmute::<&'b mut *const (), &'b mut *const libc::c_void>(original) };
    invoke(
        device,
        inode,
        symbol.to_bytes_with_nul().as_ptr().cast(),
        replacement.cast(),
        original,
    );
}

pub(crate) fn plt_hook_exclude(
    regex: impl AsRef<ffi::CStr>,
    symbol: impl AsRef<ffi::CStr>,
    invoke: impl FnOnce(*const c_char, *const c_char),
) {
    let regex = regex.as_ref();
    let symbol = symbol.as_ref();
    invoke(
        regex.to_bytes_with_nul().as_ptr().cast(),
        symbol.to_bytes_with_nul().as_ptr().cast(),
    );
}

pub(crate) fn plt_hook_commit(committed: bool) -> Result<(), ZygiskError> {
    if committed {
        Ok(())
    } else {
        Err(ZygiskError::PltHookCommitError)
    }
}

pub(crate) fn parse_state_flags(flags: u32) -> Result<crate::api::v2::StateFlags, ZygiskError> {
    match crate::api::v2::StateFlags::from_bits(flags) {
        Some(flags) => Ok(flags),
        None => Err(ZygiskError::UnrecognizedStateFlag(flags)),
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::CString, os::fd::IntoRawFd};

    use super::*;

    #[test]
    fn companion_error_does_not_take_a_callback_path() {
        let result = with_companion(|| -1, |_| panic!("callback must not run"));
        assert!(matches!(result, Err(ZygiskError::ConnectCompanionError)));
    }

    #[test]
    fn companion_takes_ownership_of_success_fd() {
        let (left, _right) = UnixStream::pair().expect("socket pair");
        let fd = left.into_raw_fd();
        let result = with_companion(|| fd, |stream| stream.peer_addr().is_ok());
        assert!(matches!(result, Ok(true)));
    }

    #[test]
    fn basic_plt_register_converts_inputs_once() {
        let regex = CString::new("/system/lib").unwrap();
        let symbol = CString::new("foo").unwrap();
        let mut original = core::ptr::null();
        plt_hook_register_basic(
            &regex,
            &symbol,
            core::ptr::null(),
            &mut original,
            |regex_ptr, symbol_ptr, replacement, original| {
                assert_eq!(
                    unsafe { core::ffi::CStr::from_ptr(regex_ptr) },
                    regex.as_c_str()
                );
                assert_eq!(
                    unsafe { core::ffi::CStr::from_ptr(symbol_ptr) },
                    symbol.as_c_str()
                );
                assert!(replacement.is_null());
                *original = core::ptr::null();
            },
        );
    }

    #[test]
    fn commit_result_maps_to_public_error() {
        assert!(matches!(plt_hook_commit(true), Ok(())));
        assert!(matches!(
            plt_hook_commit(false),
            Err(ZygiskError::PltHookCommitError)
        ));
    }
}
