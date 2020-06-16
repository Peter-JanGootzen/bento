/*
 * SPDX-License-Identifier: GPL-2.0
 * Copyright (C) 2020 Samantha Miller, Kaiyuan Zhang, Danyang Zhuo, Tom
      Anderson, Ang Chen, University of Washington
 *
 */

#![macro_use]
use kernel::raw;

pub type Condition = extern "C" fn() -> bool;

/// A macro to create a Rust wrapper around a kernel data type.
///
/// The resulting struct has one field: a pointer to a C data structure.
/// The data layout is therefore identical to a C pointer, so C functions can pass pointers to Rust
/// functions that accept these types.
///
/// Wrapper types cannot be created safely. The `from_raw` method can be used in unsafe Rust to
/// create a wrapper type given a pointer. The `get_raw` method can be used to access the pointer
/// given the wrapper type.
///
/// TODO: Maybe make this a Trait with derive instead of a macro because we can't document the
/// resulting types this way.
///
/// # Examples
///
/// ```
/// // Creates a wrapper-type for a super_block.
/// def_kernel_obj_type!(RsSuperBlock);
/// ```
#[macro_export]
macro_rules! def_kernel_obj_type {
    ($ref_name: ident) => {
        #[derive(Debug, Clone)]
        pub struct $ref_name(*const kernel::raw::c_void);
        impl $ref_name {
            pub unsafe fn from_raw(ptr: *const kernel::raw::c_void) -> $ref_name {
                $ref_name(ptr)
            }

            pub fn get_raw(&self) -> *const kernel::raw::c_void {
                self.0
            }
        }
    };
}

/// A macro for generating a getter function for a non-primitive on a wrapper type.
///
/// This will generate a Rust function that calls a C function. The name of the C function is
/// determined by the types passed into macro. The resulting C function must be defined in
/// helpers.c and exposed in the `extern` block.
///
/// For example, `def_kernel_obj_getter!(RsSuperBlock, s_bdev, super_block, RsBlockDevice);` would
/// generate a function implemented on the `RsSuperBlock` type that returns a `RsBlockDevice`. It
/// would call a C function named `rs_super_block_get_s_bdev`. A user of the `RsSuperBlock` type
/// could then call `s_bdev()` to call the getter function.
#[macro_export]
macro_rules! def_kernel_obj_getter {
    ($t_name: ty, $field_name: ident, $c_type: ident, $field_type: ident) => {
        impl $t_name {
            pub fn $field_name(&self) -> $field_type {
                use kernel::ffi::*;
                let f = concat_idents!(rs_, $c_type, _get_, $field_name);
                unsafe { $field_type(f(self.0)) }
            }
        }
    };
}

/// A macro for generating a getter function for a potentially-NULL non-primitive on a wrapper type.
///
/// This is very much like `def_kernel_obj_getter` except it is safe to use when the resulting C
/// function may return a NULL pointer or an error value cast to a pointer. This function returns
/// an `Option`, returning `None` if the returned pointer is a NULL or an error value.
#[macro_export]
macro_rules! def_kernel_nullable_obj_getter {
    ($t_name: ty, $field_name: ident, $c_type: ident, $field_type: ident) => {
        impl $t_name {
            pub fn $field_name(&self) -> Option<$field_type> {
                use crate::bindings::*;
                use kernel::ffi::*;
                let f = concat_idents!(rs_, $c_type, _get_, $field_name);
                unsafe {
                    let ptr = f(self.0);
                    match ptr.is_null() || IS_ERR(ptr as u64) {
                        true => None,
                        false => Some($field_type(ptr)),
                    }
                }
            }
        }
    };
}

/// A macro for generating a setter function for a non-primitive on a wrapper type.
///
/// This macro is very much like `def_kernel_obj_getter` except that it generates a setter function
/// instead of a getter function. The same of the setter function exposed to Rust is passed in as
/// the second argument of the macro.
#[macro_export]
macro_rules! def_kernel_obj_setter {
    ($t_name: ty, $setter_name: ident, $field_name: ident, $c_type: ident, $field_type: ident) => {
        impl $t_name {
            pub fn $setter_name(&mut self, obj: $field_type) {
                use kernel::ffi::*;
                let f = concat_idents!(rs_, $c_type, _set_, $field_name);
                unsafe {
                    f(self.0, obj.get_raw());
                }
            }
        }
    };
}

/// A macro for generating a getter function for a primitive on a wrapper type.
///
/// This function works much like `def_kernel_obj_getter` except it generates getters for primitive
/// types instead of object types.
#[macro_export]
macro_rules! def_kernel_val_getter {
    ($t_name: ty, $field_name: ident, $c_type: ident, $field_type: ty) => {
        impl $t_name {
            pub fn $field_name(&self) -> $field_type {
                use kernel::ffi::*;
                let f = concat_idents!(rs_, $c_type, _get_, $field_name);
                unsafe { f(self.0) as $field_type }
            }
        }
    };
}

/// A macro for generating a setter function for a primitive on a wrapper type.
///
/// This function works much like `def_kernel_obj_setter` except it generates setters for primitive
/// types instead of object types.
#[macro_export]
macro_rules! def_kernel_val_setter {
    ($t_name: ty, $setter_name: ident, $field_name: ident, $c_type: ident, $field_type: ty) => {
        impl $t_name {
            pub fn $setter_name(&mut self, obj: $field_type) {
                use kernel::ffi::*;
                let f = concat_idents!(rs_, $c_type, _set_, $field_name);
                unsafe {
                    f(self.0, obj);
                }
            }
        }
    };
}

/// A macro for generating both a getter and a setter for a non-primitive on a wrapper type.
///
/// This combines `def_kernel_obj_getter` and `def_kernel_obj_setter` into one macro.
#[macro_export]
macro_rules! def_kernel_obj_accessors {
    ($t_name: ty, $setter_name: ident, $field_name: ident, $c_type: ident, $field_type: ident) => {
        def_kernel_obj_getter!($t_name, $field_name, $c_type, $field_type);
        def_kernel_obj_setter!($t_name, $setter_name, $field_name, $c_type, $field_type);
    };
}

/// A macro for generating both a getter and a setter for a primitive on a wrapper type.
///
/// This combines `def_kernel_val_getter` and `def_kernel_val_setter` into one macro.
#[macro_export]
macro_rules! def_kernel_val_accessors {
    ($t_name: ty, $setter_name: ident, $field_name: ident, $c_type: ident, $field_type: ty) => {
        def_kernel_val_getter!($t_name, $field_name, $c_type, $field_type);
        def_kernel_val_setter!($t_name, $setter_name, $field_name, $c_type, $field_type);
    };
}

/// A macro for generating a mutable operation on a mutable wrapper type.
///
/// This macro will call a C-function that takes one argument (the wrapper type). The C function
/// must be defined in helpers.c and exposed in the `extern` block. This macro will expose a method
/// on the wrapper type that borrows the wrapper mutably, takes no arguments, and calls that C function.
///
/// Examples:
/// ```
/// def_kernel_obj_type!(RsBufferHead);
/// def_kobj_op!(RsBufferHead, sync_dirty_buffer, sync_dirty_buffer, i32);
///
/// // bh should be provided by C.
/// fn do_something(bh: RsBufferHead) {
///     // Calls sync_dirty_buffer(*const buffer_head bh) in the kernel
///     let ret: i32 = bh.sync_dirty_buffer();
///     ...
/// }
#[macro_export]
macro_rules! def_kobj_op {
    // TODO: extend this macro to variadic arguments
    // TODO: extend this macro to have return value
    // TODO: also add a version that does immutable borrow
    ($t_name: ty, $method_name: ident, $c_func_name: ident, $ret_type: ty) => {
        impl $t_name {
            pub fn $method_name(&mut self) -> $ret_type {
                unsafe { $c_func_name(self.get_raw()) }
            }
        }
    };
}

/// A macro for generating an immutable operation on an immutable wrapper type.
///
/// This macro works just like `def_kobj_op` except it generates a method that borrows the wrapper
/// immutably.
#[macro_export]
macro_rules! def_kobj_immut_op {
    // TODO: extend this macro to variadic arguments
    // TODO: extend this macro to have return value
    // TODO: also add a version that does immutable borrow
    ($t_name: ty, $method_name: ident, $c_func_name: ident, $ret_type: ty) => {
        impl $t_name {
            pub fn $method_name(&self) -> $ret_type {
                unsafe { $c_func_name(self.get_raw()) }
            }
        }
    };
}

extern "C" {
    pub fn printk(fmt: *const raw::c_char, ...) -> raw::c_int;

    // kmem
    pub fn __kmalloc(size: raw::c_size_t, flags: u32) -> *mut raw::c_void;
    pub fn kfree(ptr: *const raw::c_void);

    // mem: TODO: implement these in rust
    pub fn memchr(s: *const raw::c_void, c: i32, n: raw::c_size_t) -> *const raw::c_void;

    // block cache
    pub fn rs_sb_bread(sb: *const raw::c_void, blockno: u64) -> *const raw::c_void;
    pub fn __brelse(buf: *const raw::c_void);
    pub fn blkdev_issue_flush(
        bdev: *const raw::c_void,
        gfp_mask: usize,
        error_sector: *mut u64,
    ) -> isize;
    pub fn rs_super_block_get_s_bdev(sb: *const raw::c_void) -> *const raw::c_void;

    // fs
    pub fn rs_buffer_head_get_b_data(bh: *const raw::c_void) -> *const raw::c_void;
    pub fn rs_buffer_head_get_b_size(bh: *const raw::c_void) -> raw::c_size_t;

    pub fn mark_buffer_dirty(bh: *const raw::c_void);
    pub fn sync_dirty_buffer(bh: *const raw::c_void) -> i32;

    pub fn rs_get_semaphore() -> *mut raw::c_void;
    pub fn rs_put_semaphore(sem: *const raw::c_void);
    pub fn down_read(sem: *const raw::c_void);
    pub fn up_read(sem: *const raw::c_void);
    pub fn down_write(sem: *const raw::c_void);
    pub fn down_write_trylock(sem: *const raw::c_void) -> i32;
    pub fn up_write(sem: *const raw::c_void);

    // string
    pub fn strnlen(s: *const raw::c_char, max_len: u64) -> u64;
    pub fn strcmp(s1: *const raw::c_char, s2: *const raw::c_char) -> i32;

    // debugging relaed
    pub fn rs_dump_super_block(sb: *const raw::c_void);
    pub fn msleep(msecs: u32);
    pub fn rs_ndelay(usecs: u32);

    pub fn getnstimeofday64(ts: *const raw::c_void);

    pub fn rs_get_wait_queue_head() -> *mut raw::c_void;
    pub fn rs_put_wait_queue_head(wq_head: *const raw::c_void);
    pub fn rs_wake_up(wq_head: *const raw::c_void);
    pub fn rs_wait_event(wq_head: *const raw::c_void, condition: Condition);
    pub fn register_bento_fs(fs_name: *const raw::c_void, fs_ops: *const raw::c_void) -> i32;
    pub fn unregister_bento_fs(fs_name: *const raw::c_void) -> i32;
}

pub unsafe fn sb_bread(sb: *const raw::c_void, blockno: u64) -> *const raw::c_void {
    rs_sb_bread(sb, blockno)
}
