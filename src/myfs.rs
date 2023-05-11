
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

        pub fn create_file(&self, filename: [u8; 8], size: u8) {
            println!("creating {:?}: size: {size}", std::str::from_utf8(&filename).unwrap());
        }

        pub fn delete_file(&self, filename: [u8; 8]) {
            println!("deleting {:?}", std::str::from_utf8(&filename).unwrap());
        }

        pub fn ls(&self) {
            println!("ls'ing")
        }

        pub fn read(&self, filename: [u8; 8], block_num: u8) -> Option<[u8; BLOCK_SIZE]> {
            println!("reading {:?}: block #{block_num}", std::str::from_utf8(&filename).unwrap());
            None
            // [0; BLOCK_SIZE]
        }

        pub fn write(&self, filename: [u8; 8], block_num: u8, write_buf: [u8; BLOCK_SIZE]) {
            println!("writing {:?}: block #{block_num} with block = {:?}", std::str::from_utf8(&filename).unwrap(), String::from_utf8_lossy(&write_buf));
        }

        pub fn close_disk(self) {
            drop(self.disk);
        }
    }