#![feature(slice_fill)]

extern crate alloc;
//#[macro_use]
extern crate bento_utils;
extern crate core;
extern crate datablock;
extern crate fuse;
extern crate serde;
extern crate time;

#[macro_use]
pub mod xv6fs_ll;
pub mod xv6fs_file;
pub mod xv6fs_fs;
pub mod xv6fs_htree;
pub mod xv6fs_log;
pub mod xv6fs_utils;

use alloc::sync::Arc;

use std::env;
use std::ffi::OsStr;
use xv6fs_ll::Xv6FileSystem;
use xv6fs_utils::BSIZE;

use bento_utils::*;
use fuse::*;
use std::path::Path;
use time::Timespec;

impl_filesystem!(Xv6FileSystem);

fn main() {
    env_logger::init();
    let disk_name = env::args_os().nth(1).unwrap();
    let fsname_arg_str = format!("fsname={}", disk_name.to_str().unwrap());
    let fsname_arg = fsname_arg_str.as_str();
    let disk = Disk::new(disk_name.to_str().unwrap(), BSIZE as u64);
    let fs = Xv6FileSystem {
        log: None,
        sb: None,
        disk: Some(Arc::new(disk)),
        ilock_cache: None,
        icache_map: None,
        ialloc_lock: None,
        balloc_lock: None,
        diskname: Some(disk_name.to_str().unwrap().to_string()),
        provino: None,
        provino_mtime: None,
    };

    let mountpoint = env::args_os().nth(2).unwrap();
    let mut opts_arr = vec!["-o", fsname_arg];
    if let Some(arg) = env::args_os().nth(3) {
        if arg.to_str().unwrap() == "blkdev" {
            opts_arr.append(&mut vec!["-o", "blkdev"]);
        }
    }
    let options = opts_arr.iter().map(OsStr::new).collect::<Vec<&OsStr>>();

    fuse::mount(fs, &mountpoint, &options).unwrap();
}
