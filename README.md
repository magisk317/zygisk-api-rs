# Zygisk API bindings for Rust

## Compatibility
| API Version | Minimum Magisk Version | Minimum ZygiskNext Version               | Implemented |
| ----------- | ---------------------- | ---------------------------------------- | ----------- |
| v1          | 23014                  | v4-0.1.0                                 | ✅           |
| v2          | 23019                  | v4-0.1.0                                 | ✅           |
| v3          | 24300                  | v4-0.1.0                                 | ✅           |
| v4          | 25204                  | v4-0.1.0                                 | ✅           |
| v5          | 26403                  | ~~It's supported in the latest version~~ | ✅           |

## This fork

This repository is an independently maintained GitLab fork of
[`rmnscnce/zygisk-api-rs`](https://github.com/rmnscnce/zygisk-api-rs).

- Maintenance repository: `https://gitlab.com/magisk3171/zygisk-api-rs`
- Upstream baseline: `457585fa8fa32d8c394ec45ae411cc36d2711680`
- Supported API versions remain v1 through v5.
- Zygisk API v6 and v7 are not implemented or claimed to be supported.
- The public API and runtime behavior are kept compatible with upstream unless a
  change is explicitly documented and tested.

The fork is intended to provide a stable dependency for MiPushZygisk and other
Rust Zygisk modules. Dependencies are pinned by commit or release tag rather
than tracking an unreviewed moving branch. The current JNI compatibility
baseline is `jni 0.21`; a future `jni 0.22` migration will be evaluated
separately because it changes JNI wrapper types and can otherwise introduce
incompatible duplicate JNI type versions into consumers.

Maintenance work is staged as follows:

1. ABI layout checks, target compilation checks, real tests, formatting and
   Clippy validation.
2. Safe internal wrappers for unsafe FFI boundaries and consolidation of
   repeated v1-v5 bridge code, without changing the public API.
3. A separately validated JNI compatibility migration, if it is needed.

The `upstream` remote and `upstream-sync` branch are retained for bringing in
upstream changes. Every synchronization is reviewed against the C++ Zygisk
headers and validated on the supported Android targets before release.


## References
- Zygisk API
  - v1 C++ header: https://github.com/topjohnwu/Magisk/blob/b8c158828484e27e2e7d6d7cb5803e6af270dc49/native/jni/zygisk/api.hpp
  - v2 C++ header: https://github.com/topjohnwu/Magisk/blob/06531f6d06a73b4770762964e41201b9f157923b/native/jni/zygisk/api.hpp
  - v3 C++ header: https://github.com/topjohnwu/Magisk/blob/1565bf5442e10b0f1b1908856f21e45703baa29a/native/src/zygisk/api.hpp
  - v4 C++ header: https://github.com/topjohnwu/Magisk/blob/65c18f9c09afa80774867b6ef26622ed7b4e0c96/native/src/core/zygisk/api.hpp
  - v5 C++ header: https://github.com/topjohnwu/Magisk/blob/e35925d520b5fab3acc96c1f137f951edca06760/native/src/core/zygisk/api.hpp
