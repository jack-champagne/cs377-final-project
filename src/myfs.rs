
use std::{fs::{File, OpenOptions}, io::{Seek, Read, Write}};

pub const BLOCK_SIZE: usize = 1024;
const FREE_BLOCK_SIZE: usize = 128;
const MAX_INODES: usize = 16;
const IDXNODE_SIZE: usize = std::mem::size_of::<IDXNode>();

#[derive(Debug)]
struct IDXNode {
    name : [u8; 8],
    size: u8,
    block_pointers: [u8; 8],
    used: u8,
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

    fn get_inode(&mut self, inode_index: usize) -> IDXNode {
        let mut inode_buffer = [0u8; IDXNODE_SIZE];
        self.disk.seek(std::io::SeekFrom::Start((FREE_BLOCK_SIZE + IDXNODE_SIZE * inode_index) as u64)).unwrap();
        self.disk.read(&mut inode_buffer).unwrap();
            unsafe {
                std::mem::transmute::<[u8; IDXNODE_SIZE], IDXNode>(inode_buffer)
            }
    }

    fn find_inode_cond(&mut self, f: impl Fn(&IDXNode) -> bool) -> Result<IDXNode, &str> {
        for i in 0..MAX_INODES {
            let inode = self.get_inode(i);
            if f(&inode) {
                return Ok(inode)
            }
        }
        Err("Could not find inode")
    }

    fn write_free_block_list(&mut self, free_block_list: [u8; FREE_BLOCK_SIZE]) {
        self.disk.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.disk.write(&free_block_list).unwrap();
    }

    fn write_inode(&mut self, inode: IDXNode) {
        let inode_buffer = unsafe {
            std::mem::transmute::<IDXNode, [u8; IDXNODE_SIZE]>(inode)
        };
        self.disk.seek(std::io::SeekFrom::Current(-(IDXNODE_SIZE as i64))).unwrap();
        self.disk.write(&inode_buffer).unwrap();
    }

    fn get_free_block_list(&mut self) -> [u8; FREE_BLOCK_SIZE] {
        self.disk.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut free_block_list: [u8; FREE_BLOCK_SIZE] = [0; FREE_BLOCK_SIZE];
        self.disk.read(&mut free_block_list).unwrap();
        return free_block_list
        
    }

    pub fn create_file(&mut self, filename: [u8; 8], size: u8) -> Result<(), String>{
        // println!("creating {:?}: size: {size}", std::str::from_utf8(&filename).unwrap());
        let mut free_block_list = self.get_free_block_list();
        let available_blocks = free_block_list.iter().fold(0, |acc, &x| if x == 0 { acc + 1 } else { acc });
        if available_blocks < size {
            return Err(String::from("Not enough free blocks"));
        }
        if size > 8 {
            return Err(String::from(format!("Max blocks per file is 8, not {}", size)))
        }
        
        let mut inode = self.find_inode_cond(|i| i.used == 0)?;

        inode.used = 1;
        inode.name = filename;
        inode.size = size;

        // FIND FREE BLOCKS
        let mut blocks_allocated = 0;
        let mut i = 0;
        while i < FREE_BLOCK_SIZE as u8 && blocks_allocated < size as usize {
            if free_block_list[i as usize] == 0 {
                free_block_list[i as usize] = 1;
                inode.block_pointers[blocks_allocated] = i;
                blocks_allocated += 1;
            }
            i += 1;
        }

        self.write_inode(inode);
        self.write_free_block_list(free_block_list);
        return Ok(())
    }

    pub fn delete_file(&mut self, filename: [u8; 8]) {
        let mut free_block_list = self.get_free_block_list();
        if let Ok(mut inode) = self.find_inode_cond(|x| x.name == filename) {
            for i in 0..inode.size {
                free_block_list[inode.block_pointers[i as usize] as usize] = 0;
            }
            inode.used = 0;
            self.write_inode(inode);
            self.write_free_block_list(free_block_list);
        }
    }

    pub fn ls(&mut self) {
        for i in 0..MAX_INODES {
            let inode = self.get_inode(i);
            if inode.used == 1 {
                println!("{}", std::str::from_utf8(&inode.name).unwrap());
            }
        }
    }

    pub fn read(&mut self, filename: [u8; 8], block_num: u8) -> Option<[u8; BLOCK_SIZE]> {
        if let Ok(inode) = self.find_inode_cond(|x| x.name == filename) {
            let block = inode.block_pointers[block_num as usize];
            let mut buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            self.disk.seek(std::io::SeekFrom::Start((FREE_BLOCK_SIZE + BLOCK_SIZE * block as usize) as u64)).unwrap();
            self.disk.read(&mut buf).unwrap();
            return Some(buf)
        }
        None
    }

    pub fn write(&mut self, filename: [u8; 8], block_num: u8, write_buf: [u8; BLOCK_SIZE]) {
        if let Ok(inode) = self.find_inode_cond(|x| x.name == filename) {
            let block = inode.block_pointers[block_num as usize];
            self.disk.seek(std::io::SeekFrom::Start((FREE_BLOCK_SIZE + BLOCK_SIZE * block as usize) as u64)).unwrap();
            self.disk.write(&write_buf).unwrap();
        }
    }

    pub fn close_disk(self) {
        drop(self.disk);
    }
}
// Written by Jack Champagne

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    fn setup() {

        Command::new("./create_fs")
        .arg("disk0")
        .stdout(Stdio::null())
        .spawn()
        .expect("sh command failed to start");
    }

    #[test]
    #[should_panic]
    fn bad_fs() {
        setup();
        let mut my_fs = MyFileSystem::new("diskL");
        my_fs.create_file([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8], 7).unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_file() {
        setup();
        let mut my_fs = MyFileSystem::new("disk0");
        my_fs.create_file([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8], 100).unwrap();
    }

    #[test]
    fn good_file_ops() {
        setup();
        let mut my_fs = MyFileSystem::new("disk0");
        let my_filename = crate::get_filename_array("testfile");
        my_fs.create_file(my_filename, 8).unwrap();
        my_fs.write(my_filename, 1, [12; BLOCK_SIZE]);
        let buf = [0; BLOCK_SIZE];
        assert!(my_fs.read(my_filename, 0).unwrap() == buf);
        assert!(my_fs.read(my_filename, 1).unwrap() != buf);
    }
}