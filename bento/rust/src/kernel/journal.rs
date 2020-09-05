/*
 * SPDX-License-Identifier: GPL-2.0
 * Copyright (C) 2020 Samantha Miller, Kaiyuan Zhang, Danyang Zhuo, Tom
      Anderson, Ang Chen, University of Washington
 *
 */

use core::cell::UnsafeCell;

use kernel::ffi::*;
use kernel::kobj::*;
use kernel::raw::*;

use core::cell::RefCell;
use alloc::sync::Arc;
use alloc::vec::Vec;

use kernel::fs::*;

use crate::bento_utils::Disk;

/// Wrapper around the kernel `journal_t`.
#[derive(Debug)]
pub struct Journal {
    journal: UnsafeCell<RsJournal>,
}

/// Wrapper around the kernel `handle_t`.
pub struct Handle {
    handle: UnsafeCell<RsHandle>,
    requested: u32,
    blocks: RefCell<Vec<u64>>,
}

impl Journal {
    pub fn new(bdev: &BlockDevice, fs_dev: &BlockDevice, start: u64, len: i32, bsize: i32) -> Option<Journal> {
        println!("initializing journal");

        let journal;
        unsafe {
            journal = rs_jbd2_journal_init_dev(bdev.bdev.get_raw() as *const c_void, 
                                                fs_dev.bdev.get_raw() as *const c_void, 
                                                start, 
                                                len, 
                                                bsize);
        }
        if journal.is_null() {
            return None;
        } else {
            unsafe {
                // TODO call jbd2_journal_load
                if rs_jbd2_journal_load(journal) != 0 {
                    return None;
                }
                rs_jbd2_journal_set_barrier(journal);
                rs_jbd2_journal_set_async_commit(journal);

                return Some(Journal { 
                    journal: UnsafeCell::new(RsJournal::from_raw(journal as *const c_void)),
                });
            }
        }
    }

    pub fn new_from_disk(disk: Arc<Disk>, fs_disk: Arc<Disk>, start: u64, len: i32, bsize: i32) -> Option<Journal> {
        Journal::new(&disk.bdev, &fs_disk.bdev, start, len, bsize)
    }

    // begin transaction of size blocks
    pub fn begin_op(&self, blocks: u32) -> Handle {
        let handle;
        //println!("begin {}", blocks);
        unsafe {
            handle = rs_jbd2_journal_start((*self.journal.get()).get_raw() as *const c_void, blocks as i32)
        }
        if handle.is_null() {
            panic!("transaction begin failed")
        } else {
            unsafe {
                return Handle {
                    handle: UnsafeCell::new(RsHandle::from_raw(handle as *const c_void)),
                    requested: blocks,
                    blocks: RefCell::new(Vec::new()),
                };
            }
        }
    }

    // force completed transactions to write to disk
    pub fn force_commit(&self) -> i32 {
        unsafe {
            return rs_jbd2_journal_force_commit((*self.journal.get()).get_raw() as *const c_void);
        }
    }

    pub fn destroy(&self) {
        println!("cleaning up journal");
        unsafe {
            //self.force_commit();
            rs_jbd2_journal_destroy((*self.journal.get()).get_raw() as *const c_void);
        }
    }
}

impl Handle {
    // notify intent to modify BufferHead as a part of this transaction
    pub fn get_write_access(&self, bh: &BufferHead) -> i32 {
        let vec: &mut Vec<u64> = &mut self.blocks.borrow_mut();
        if vec.contains(&bh.blocknr()) {
            return 0;
        }
        unsafe {
            return rs_jbd2_journal_get_write_access((*self.handle.get()).get_raw() as *const c_void, bh.get_raw());
        }
    }

    pub fn get_create_access(&self, bh: &BufferHead) -> i32 {
        let vec: &mut Vec<u64> = &mut self.blocks.borrow_mut();
        if vec.contains(&bh.blocknr()) {
            return 0;
        }
        unsafe {
            return rs_jbd2_journal_get_create_access((*self.handle.get()).get_raw() as *const c_void, bh.get_raw());
        }
    }

    // register a block as part of the transaction associated with this handle
    pub fn journal_write(&self, bh: &mut BufferHead) -> i32 {
        let blocknr = bh.blocknr();
        let vec: &mut Vec<u64> = &mut self.blocks.borrow_mut();
        if !vec.contains(&blocknr) {
            vec.push(blocknr);
        }
        if vec.len() > self.requested as usize {
            println!("too many unique blocks written: {} / {}", vec.len(), self.requested);
        }

        unsafe {
            return rs_jbd2_journal_dirty_metadata((*self.handle.get()).get_raw() as *const c_void, bh.get_raw());
        }
    }
}

// ends transaction
impl Drop for Handle {
    fn drop(&mut self) {
        let res;
        unsafe {
            res = rs_jbd2_journal_stop((*self.handle.get()).get_raw() as *const c_void);
        }
        if res == 0 {
             ()
        } else {
             println!("some log transaction was aborted");
             loop {};
        }
    }
}

unsafe impl Sync for Journal {}
