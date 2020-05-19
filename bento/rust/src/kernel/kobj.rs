use kernel;
use kernel::errno::Error;
use kernel::mem;
use kernel::raw::*;

use core::slice;

// /// A wrapper around the kernel `super_block` type.
def_kernel_obj_type!(RsSuperBlock);
// /// A wrapper around the kernel `buffer_head` type.
// ///
// /// Acquired by using `sb_bread_rust`. Since each bread must be accompanied by an associated brelse
// /// to release the buffer, this calls `brelse` on `drop`.
def_kernel_obj_type!(RsBufferHead);
// /// A wrapper around the kernel `semaphore` type.
def_kernel_obj_type!(RsRwSemaphore);
// /// A wrapper around the kernel `wait_queue_head` type.
def_kernel_obj_type!(RsWaitQueueHead);
// /// A wrapper around the kernel `block_device` type.
def_kernel_obj_type!(RsBlockDevice);

def_kernel_val_getter!(RsBufferHead, b_data, buffer_head, *const c_void);
def_kernel_val_getter!(RsBufferHead, b_size, buffer_head, c_size_t);

use kernel::ffi::*;

def_kernel_obj_getter!(RsSuperBlock, s_bdev, super_block, RsBlockDevice);
def_kobj_op!(RsSuperBlock, dump, rs_dump_super_block, ());

def_kobj_op!(RsBufferHead, brelse, __brelse, ());
def_kobj_op!(RsBufferHead, mark_buffer_dirty, mark_buffer_dirty, ());
def_kobj_op!(RsBufferHead, sync_dirty_buffer, sync_dirty_buffer, i32);

def_kobj_immut_op!(RsRwSemaphore, down_read, down_read, ());
def_kobj_immut_op!(RsRwSemaphore, up_read, up_read, ());
def_kobj_immut_op!(RsRwSemaphore, down_write, down_write, ());
def_kobj_immut_op!(RsRwSemaphore, down_write_trylock, down_write_trylock, i32);
def_kobj_immut_op!(RsRwSemaphore, up_write, up_write, ());
def_kobj_op!(RsRwSemaphore, put, rs_put_semaphore, ());

def_kobj_immut_op!(RsWaitQueueHead, wake_up, rs_wake_up, ());

impl Drop for RsBufferHead {
    fn drop(&mut self) {
        self.brelse();
    }
}

impl Drop for RsRwSemaphore {
    fn drop(&mut self) {
        self.put();
    }
}

//#[repr(C)]
//pub struct QStr {
//    hash: u32,
//    len: u32,
//    name: *const c_char,
//}
//
//impl QStr {
//    pub fn len(&self) -> usize {
//        self.len as usize
//    }
//
//    pub fn get_ref(&self) -> &[u8] {
//        unsafe { core::slice::from_raw_parts(self.name as *const u8, self.len()) }
//    }
//}

/// A Rust representation of a C-style, null-terminated string.
///
/// Modeled after std::ffi::CStr.
pub struct CStr {
    inner: *const c_char,
}

impl CStr {
    /// Return the pointer representation of the CStr
    pub fn to_raw(&self) -> *const c_char {
        self.inner
    }

    /// Calculate the length of the CStr.
    pub fn len(&self) -> usize {
        let mut i = 0;
        let mut ptr = self.inner;
        unsafe {
            while *ptr != 0 {
                i += 1;
                ptr = ptr.offset(1);
            }
        }
        return i;
    }

    /// Convert the CStr into a `u8` slice.
    pub fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.inner as *const u8, self.len()) }
    }

    /// Create a CStr from a `u8` slice.
    ///
    /// Will return an error if the byte array doesn't contain a null character.
    pub fn from_bytes_with_nul(bytes: &[u8]) -> Result<CStr, Error> {
        let mut nul_pos = None;
        for (iter, byte) in bytes.iter().enumerate() {
            if *byte == 0 {
                nul_pos = Some(iter);
                break;
            }
        }
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != bytes.len() {
                return Err(Error::EIO);
            }
            Ok(CStr {
                inner: bytes.as_ptr() as *const i8,
            })
        } else {
            return Err(Error::EIO);
        }
    }
}

impl RsBufferHead {
    /// Return the associated data as a `kernel::MemContainer<c_uchar>`.
    pub fn get_buffer_data(&self) -> mem::MemContainer<c_uchar> {
        let b_data = self.b_data();
        let size = self.b_size();
        unsafe {
            return mem::MemContainer::new_from_raw(b_data as *mut c_uchar, size as usize);
        }
    }

    /// Return the associated data as a `c_uchar` slice.
    pub fn get_data(&self) -> &[c_uchar] {
        let b_data = self.b_data();
        let size = self.b_size();
        unsafe {
            return slice::from_raw_parts::<c_uchar>(b_data as *mut c_uchar, size as usize);
        }
    }

    /// Return the associated data as a mutable `c_uchar` slice.
    pub fn get_mut_data(&mut self) -> &mut [c_uchar] {
        let b_data = self.b_data();
        let size = self.b_size();
        unsafe {
            return slice::from_raw_parts_mut::<c_uchar>(b_data as *mut c_uchar, size as usize);
        }
    }
}

impl RsWaitQueueHead {
    /// Block waiting on an event.
    ///
    /// This calls the `wait_event` function in the kernel. The function will unblock when the
    /// condition may be true. Users should check the condition again after unblocking.
    pub unsafe fn wait_event(&self, condition: Condition) {
        rs_wait_event(self.get_raw() as *const c_void, condition);
    }
}
