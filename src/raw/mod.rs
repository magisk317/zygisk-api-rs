use core::{marker::PhantomData, ptr::NonNull};

use jni::EnvUnowned;
use libc::c_long;

use crate::{ZygiskModule, impl_sealing::Sealed};

macro_rules! define_callback_trampolines {
    ($version:ty, $raw_module:ty, $app_args:ty, $server_args:ty) => {
        extern "C" fn pre_app_specialize<'a>(module: &mut $raw_module, args: &'a mut $app_args) {
            module.dispatch.pre_app_specialize(
                crate::api::ZygiskApi::<$version>(module.api_table),
                // SAFETY: The stored pointer belongs to the current JNI-attached thread.
                unsafe { EnvUnowned::from_raw(module.jni_env.as_raw()) },
                args,
            );
        }

        extern "C" fn post_app_specialize<'a>(module: &mut $raw_module, args: &'a $app_args) {
            module.dispatch.post_app_specialize(
                crate::api::ZygiskApi::<$version>(module.api_table),
                // SAFETY: See `pre_app_specialize`.
                unsafe { EnvUnowned::from_raw(module.jni_env.as_raw()) },
                args,
            );
        }

        extern "C" fn pre_server_specialize<'a>(
            module: &mut $raw_module,
            args: &'a mut $server_args,
        ) {
            module.dispatch.pre_server_specialize(
                crate::api::ZygiskApi::<$version>(module.api_table),
                // SAFETY: See `pre_app_specialize`.
                unsafe { EnvUnowned::from_raw(module.jni_env.as_raw()) },
                args,
            );
        }

        extern "C" fn post_server_specialize<'a>(module: &mut $raw_module, args: &'a $server_args) {
            module.dispatch.post_server_specialize(
                crate::api::ZygiskApi::<$version>(module.api_table),
                // SAFETY: See `pre_app_specialize`.
                unsafe { EnvUnowned::from_raw(module.jni_env.as_raw()) },
                args,
            );
        }
    };
}

macro_rules! define_module_abi {
    ($version:ty, $module:expr, $raw_module:ty, $app_args:ty, $server_args:ty) => {{
        define_callback_trampolines!($version, $raw_module, $app_args, $server_args);
        ModuleAbi {
            api_version: <$version as ZygiskRaw<'_>>::API_VERSION,
            this: $module,
            pre_app_specialize_fn: pre_app_specialize,
            post_app_specialize_fn: post_app_specialize,
            pre_server_specialize_fn: pre_server_specialize,
            post_server_specialize_fn: post_server_specialize,
        }
    }};
}

pub(crate) use define_module_abi;

macro_rules! forward_register_module {
    ($table:expr) => {{
        // SAFETY: The table comes from the active Zygisk runtime and its base
        // prefix is present in every supported API version.
        unsafe { &*$table.0 }.base.register_module_fn
    }};
}

pub(crate) use forward_register_module;

pub mod v1;
pub mod v2;
pub mod v3;
pub mod v4;
pub mod v5;

#[doc(hidden)]
pub struct RawModule<'a, Version>
where
    Version: ZygiskRaw<'a> + 'a + ?Sized,
{
    #[doc(hidden)]
    pub dispatch: &'a (dyn ZygiskModule<Api = Version> + 'a),
    #[doc(hidden)]
    pub api_table: ApiTableRef<'a, Version>,
    #[doc(hidden)]
    pub jni_env: EnvUnowned<'a>,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ApiTableRef<'a, Version>(
    pub(crate) *const <Version as ZygiskRaw<'a>>::ApiTable,
    PhantomData<&'a Version>,
)
where
    Version: ZygiskRaw<'a> + 'a + ?Sized;

impl<'a, Version> ApiTableRef<'a, Version>
where
    Version: ZygiskRaw<'a> + 'a,
{
    /// Creates a typed view over the API table supplied by Zygisk.
    ///
    /// # Safety
    ///
    /// `api_tbl` must be non-null, point to a valid table for `Version`, and
    /// remain readable for the returned lifetime. The table must be supplied
    /// by the active Zygisk runtime rather than an arbitrary Rust value.
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn from_raw(api_tbl: *const <Version as ZygiskRaw<'a>>::ApiTable) -> Self {
        Self(api_tbl, PhantomData)
    }
}

#[repr(C)]
pub struct ModuleAbi<'a, Version>
where
    Version: ZygiskRaw<'a> + 'a + ?Sized,
{
    pub(crate) api_version: c_long,
    pub(crate) this: &'a mut RawModule<'a, Version>,

    pub(crate) pre_app_specialize_fn: for<'c, 'd> extern "C" fn(
        &'d mut RawModule<'c, Version>,
        &'c mut <Version as ZygiskRaw<'c>>::AppSpecializeArgs,
    ),
    pub(crate) post_app_specialize_fn: for<'c, 'd> extern "C" fn(
        &'d mut RawModule<'c, Version>,
        &'c <Version as ZygiskRaw<'c>>::AppSpecializeArgs,
    ),
    pub(crate) pre_server_specialize_fn: for<'c, 'd> extern "C" fn(
        &'d mut RawModule<'c, Version>,
        &'c mut <Version as ZygiskRaw<'c>>::ServerSpecializeArgs,
    ),
    pub(crate) post_server_specialize_fn: for<'c, 'd> extern "C" fn(
        &'d mut RawModule<'c, Version>,
        &'c <Version as ZygiskRaw<'c>>::ServerSpecializeArgs,
    ),
}

/// Opaque type representing the API instance handle pointer
#[repr(transparent)]
pub(crate) struct Instance(());

#[repr(C)]
pub(crate) struct BaseApi<V>
where
    for<'a> V: ZygiskRaw<'a>,
{
    pub(crate) this: NonNull<Instance>,
    pub(crate) register_module_fn:
        for<'a> unsafe extern "C" fn(ApiTableRef<V>, ModuleAbiRef<'a, V>) -> bool,
}

#[repr(transparent)]
pub struct ModuleAbiRef<'a, Version>(
    pub(crate) *mut ModuleAbi<'a, Version>,
    PhantomData<&'a Version>,
)
where
    Version: ZygiskRaw<'a> + ?Sized;

impl<'a, Version> ModuleAbiRef<'a, Version>
where
    Version: ZygiskRaw<'a>,
{
    /// Creates the opaque ABI handle passed back to Zygisk.
    ///
    /// # Safety
    ///
    /// `module_abi` must be non-null, point to a fully initialized
    /// [`ModuleAbi`] for `Version`, and remain valid for the returned
    /// lifetime. The pointer must not be used after its backing storage is
    /// released or moved.
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn from_raw(module_abi: *mut ModuleAbi<'a, Version>) -> Self {
        Self(module_abi, PhantomData)
    }
}

pub trait ZygiskRaw<'a>
where
    Self: Sealed + 'a,
{
    const API_VERSION: c_long;
    type ApiTable: 'a;
    type AppSpecializeArgs: 'a;
    type ServerSpecializeArgs: 'a;

    fn abi_from_module(module: &'a mut RawModule<'a, Self>) -> ModuleAbi<'a, Self>;

    fn register_module_fn(
        table: ApiTableRef<'a, Self>,
    ) -> for<'b> unsafe extern "C" fn(ApiTableRef<'a, Self>, ModuleAbiRef<'b, Self>) -> bool;
}

#[cfg(test)]
mod abi_layout_tests {
    use core::{mem, mem::offset_of};

    use libc::c_long;

    use super::ModuleAbi;
    use crate::api::{V1, V2, V3, V4, V5};

    macro_rules! assert_module_abi_layout {
        ($version:ty) => {{
            let pointer_size = mem::size_of::<*const ()>();
            let long_size = mem::size_of::<c_long>();
            let pointer_offset = (long_size + pointer_size - 1) & !(pointer_size - 1);

            assert_eq!(offset_of!(ModuleAbi<'static, $version>, api_version), 0);
            assert_eq!(
                offset_of!(ModuleAbi<'static, $version>, this),
                pointer_offset
            );

            let mut next_offset = pointer_offset + pointer_size;
            assert_eq!(
                offset_of!(ModuleAbi<'static, $version>, pre_app_specialize_fn),
                next_offset
            );
            next_offset += pointer_size;
            assert_eq!(
                offset_of!(ModuleAbi<'static, $version>, post_app_specialize_fn),
                next_offset
            );
            next_offset += pointer_size;
            assert_eq!(
                offset_of!(ModuleAbi<'static, $version>, pre_server_specialize_fn),
                next_offset
            );
            next_offset += pointer_size;
            assert_eq!(
                offset_of!(ModuleAbi<'static, $version>, post_server_specialize_fn),
                next_offset
            );
            next_offset += pointer_size;

            assert_eq!(mem::size_of::<ModuleAbi<'static, $version>>(), next_offset);
            assert_eq!(
                mem::align_of::<ModuleAbi<'static, $version>>(),
                pointer_size
            );
        }};
    }

    #[test]
    fn module_abi_layout_is_pointer_stable_for_all_supported_versions() {
        assert_module_abi_layout!(V1);
        assert_module_abi_layout!(V2);
        assert_module_abi_layout!(V3);
        assert_module_abi_layout!(V4);
        assert_module_abi_layout!(V5);
    }

    #[test]
    fn api_versions_are_monotonic_and_stop_at_v5() {
        assert_eq!(<V1 as super::ZygiskRaw<'static>>::API_VERSION, 1);
        assert_eq!(<V2 as super::ZygiskRaw<'static>>::API_VERSION, 2);
        assert_eq!(<V3 as super::ZygiskRaw<'static>>::API_VERSION, 3);
        assert_eq!(<V4 as super::ZygiskRaw<'static>>::API_VERSION, 4);
        assert_eq!(<V5 as super::ZygiskRaw<'static>>::API_VERSION, 5);
    }
}
