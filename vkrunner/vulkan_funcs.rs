// vkrunner
//
// Copyright (C) 2016, 2023 Neil Roberts
// Copyright (C) 2018 Intel Corporation
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice (including the next
// paragraph) shall be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::vk;
use crate::util;
use std::ffi::{c_void, c_int, c_char, CString};
use std::mem;
use std::fmt;

/// Offset of the pNext member of the structs that can be chained.
/// There doesn’t seem to be a nice equivalent to offsetof in Rust so
/// this is just trying to replicate the C struct alignment rules.
pub const NEXT_PTR_OFFSET: usize = util::align(
    mem::size_of::<vk::VkStructureType>(),
    mem::align_of::<*mut std::os::raw::c_void>(),
);
/// Offset of the first VkBool32 field in the features structs.
pub const FIRST_FEATURE_OFFSET: usize = util::align(
    NEXT_PTR_OFFSET + mem::size_of::<*mut std::os::raw::c_void>(),
    mem::align_of::<vk::VkBool32>(),
);

pub type GetInstanceProcFunc = unsafe extern "C" fn(
    func_name: *const c_char,
    user_data: *const c_void,
) -> *const c_void;

#[derive(Debug)]
pub enum Error {
    OpenLibraryFailed(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::OpenLibraryFailed(lib) => write!(f, "Error opening {}", lib)
        }
    }
}

include!{"vulkan_funcs_data.rs"}

struct LoaderFunc {
    lib_vulkan: *const c_void,
    lib_vulkan_is_fake: bool,
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
}

// Per-thread override for the get_instance_proc_address function.
// This is only used in unit tests to implement a fake Vulkan driver.
#[cfg(test)]
thread_local! {
    static LOADER_FUNC_OVERRIDE:
    std::cell::Cell<Option<LoaderFunc>> = std::cell::Cell::new(None);
}

impl Library {
    fn get_loader_func(
    ) -> Result<LoaderFunc, Error> {
        // Override for unit tests. If an override function is set
        // then we will return that instead. `take` is called on it so
        // that it will only be used once and subsequent calls will
        // revert back to the normal mechanism.
        #[cfg(test)]
        if let Some(loader_func) = LOADER_FUNC_OVERRIDE.with(|f| f.take()) {
            return Ok(loader_func);
        }

        #[cfg(unix)]
        {
            extern "C" {
                fn dlopen(name: *const c_char, flags: c_int) -> *const c_void;
                fn dlsym(
                    lib: *const c_void,
                    name: *const c_char
                ) -> *const c_void;
            }

            let lib_name;

            #[cfg(target_os = "android")]
            {
                lib_name = "libvulkan.so";
            }
            #[cfg(not(target_os = "android"))]
            {
                lib_name = "libvulkan.so.1";
            }

            let lib_name_c = CString::new(lib_name).unwrap();

            let lib = unsafe {
                dlopen(lib_name_c.as_ptr(), 1)
            };

            if lib.is_null() {
                return Err(Error::OpenLibraryFailed(lib_name));
            }

            let get_instance_proc_addr = unsafe {
                std::mem::transmute(dlsym(
                    lib,
                    "vkGetInstanceProcAddr\0".as_ptr().cast()
                ))
            };

            return Ok(LoaderFunc {
                lib_vulkan: lib,
                lib_vulkan_is_fake: false,
                get_instance_proc_addr
            });
        }

        #[cfg(windows)]
        {
            extern "system" {
                fn LoadLibraryA(
                    filename: *const c_char,
                ) -> *mut c_void;
                fn GetProcAddress(
                    handle: *mut c_void,
                    func_name: *const c_char,
                ) -> *const c_void;
            }

            let lib_name = "vulkan-1.dll";

            let lib = unsafe {
                let c_lib_name = CString::new(lib_name).unwrap();
                LoadLibraryA(c_lib_name.as_ptr())
            };

            if lib.is_null() {
                return Err(Error::OpenLibraryFailed(lib_name));
            }

            let get_instance_proc_addr = unsafe {
                std::mem::transmute(GetProcAddress(
                    lib,
                    "vkGetInstanceProcAddr\0".as_ptr().cast()
                ))
            };

            return Ok(LoaderFunc {
                lib_vulkan: lib,
                lib_vulkan_is_fake: false,
                get_instance_proc_addr
            });
        }

        #[cfg(not(any(unix,windows)))]
        todo!(
            "library opening on platforms other than Unix and Windows is \
             not yet implemented"
        );
    }

    pub fn new() -> Result<Library, Error> {
        let LoaderFunc {
            lib_vulkan,
            lib_vulkan_is_fake,
            get_instance_proc_addr,
        } = Library::get_loader_func()?;

        Ok(Library {
            lib_vulkan,
            lib_vulkan_is_fake,
            vkGetInstanceProcAddr: get_instance_proc_addr,
            vkCreateInstance: unsafe {
                std::mem::transmute(get_instance_proc_addr.unwrap()(
                    std::ptr::null_mut(),
                    "vkCreateInstance\0".as_ptr().cast()
                ))
            },
            vkEnumerateInstanceExtensionProperties: unsafe {
                std::mem::transmute(get_instance_proc_addr.unwrap()(
                    std::ptr::null_mut(),
                    "vkEnumerateInstanceExtensionProperties\0".as_ptr().cast()
                ))
            }
        })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            extern "C" {
                fn dlclose(lib: *const c_void) -> *const c_void;
            }

            if !self.lib_vulkan_is_fake {
                unsafe { dlclose(self.lib_vulkan) };
            }
        }

        #[cfg(windows)]
        {
            extern "system" {
                fn FreeLibrary(handle: *const c_void) -> c_int;
            }

            if !self.lib_vulkan_is_fake {
                unsafe { FreeLibrary(self.lib_vulkan) };
            }
        }

        #[cfg(not(any(unix,windows)))]
        todo!(
            "library closing on platforms other than Windows and Unix is not \
             yet implemented"
        );
    }
}

/// Helper function to temporarily replace the `vkGetInstanceProcAddr`
/// function that will be used for the next call to [Library::new].
/// The override will only be used once and subsequent calls to
/// [Library::new] will revert back to trying to open the Vulkan
/// library. The override is per-thread so it is safe to use in a
/// multi-threaded testing environment. This function is only
/// available in test build configs and it is intended to help make a
/// fake Vulkan driver for unit tests.
#[cfg(test)]
pub fn override_get_instance_proc_addr(
    lib: *const c_void,
    proc: vk::PFN_vkGetInstanceProcAddr,
) {
    LOADER_FUNC_OVERRIDE.with(|f| f.replace(Some(LoaderFunc {
        lib_vulkan: lib,
        lib_vulkan_is_fake: true,
        get_instance_proc_addr: proc,
    })));
}
