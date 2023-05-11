
use std::fs::{File, OpenOptions};

    pub const BLOCK_SIZE: usize = 1024;
    const FREE_BLOCK_SIZE: usize = 128;
    const MAX_INODES: usize = 16;

    struct IDXNode {
        name : [u8; 8],
        size: usize,
        block_pointers: [i32; 8],
        used: i32,
    }

    pub struct MyFileSystem {
        disk: File,
    }

    impl MyFileSystem {
        pub fn new(disk_name: &str) -> MyFileSystem {
            MyFileSystem { 
                disk: match OpenOptions::new().read(true).write(true).open(&disk_name) {
                    Ok(disk) => disk,
                    Err(_) => panic!("Could not open disk: {}", &disk_name),
                }
            }
        }
    }