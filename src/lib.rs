pub mod myfs {
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Seek, Write},
    };

    /*
    Equivalent to `idxNode` struct in original implementation however
    to have equivalent byte representation after a std::mem::transmute,
    it is necessary to define a repr, hence #[repr(C)]. Additionally,
    the original project has the block_pointers field as an array of ints
    even though there are only 128 blocks total, so it really should be an
    array of u8 instead. I have opted to keep the original project format
    for compatibility.
    */
    #[repr(C)]
    struct IDXNode {
        name: [u8; 8],
        size: u8,
        block_pointers: [u32; 8],
        used: u8,
    }

    /*
    The largest adaptation between Rust and C is the restructuring of the
    fs class into a struct that impl what was formerly methods on the class
    object. The requires mutable references to self as each one of these
    functions on the struct mutates the internal state of the struct itself.
    Here, with as faithful as an adaptation as possible to the original, the
    filesystem is a wrapper around a file stream.
    */
    pub struct MyFileSystem {
        disk: File,
    }

    pub const BLOCK_SIZE: usize = 1024;
    const FREE_BLOCK_SIZE: usize = 128;
    const MAX_INODES: usize = 16;
    const IDXNODE_SIZE: usize = std::mem::size_of::<IDXNode>();

    impl MyFileSystem {
        /// Simply the equivalent of the constructor in Rust. Work with this
        /// filesystem on an existing file, we can create an instance:
        ///
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// ```
        pub fn new(disk_name: &str) -> MyFileSystem {
            MyFileSystem {
                disk: match OpenOptions::new().read(true).write(true).open(&disk_name) {
                    Ok(disk) => disk,
                    Err(_) => panic!("Could not open disk: {}", &disk_name),
                },
            }
        }

        /// Creates a file in the filesystem with name capped at 8 bytes, and size can range from 0 to 8
        /// This will check to see if creating the file is possible and will return an Err variant if not
        /// Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// let filename: [u8; 8] = [102, 105, 108, 101, 49, 0, 0, 0]; //file1 as [u8; 8]
        /// my_file_system.create_file(filename, 8);
        /// my_file_system.ls();
        /// ```
        ///
        /// This will output the following:
        /// ```text
        /// file1
        /// ```
        pub fn create_file(&mut self, filename: [u8; 8], size: u8) -> Result<(), String> {
            if size > 8 {
                return Err(String::from(format!(
                    "Max blocks per file is 8, not {}",
                    size
                )));
            }

            let mut free_block_list = self.get_free_block_list();
            let available_blocks =
                free_block_list
                    .iter()
                    .fold(0, |acc, &x| if x == 0 { acc + 1 } else { acc });
            if available_blocks < size {
                return Err(String::from("Not enough free blocks"));
            }

            // Find an unused inode if one exists, otherwise return Err
            let mut inode = self.get_first_inode_conditional_on(|i| i.used == 0)?;
            inode.used = 1;
            inode.name = filename;
            inode.size = size;

            // Allocate available blocks for file into inode's block_pointers array.
            let mut blocks_allocated = 0;
            let mut i = 0;
            while i < FREE_BLOCK_SIZE as u32 && blocks_allocated < size as usize {
                if free_block_list[i as usize] == 0 {
                    free_block_list[i as usize] = 1;
                    inode.block_pointers[blocks_allocated] = i;
                    blocks_allocated += 1;
                }
                i += 1;
            }

            self.write_inode(inode);
            self.write_free_block_list(free_block_list);
            return Ok(());
        }

        /// Deletes a file from the filesystem by marking the inode as unused and marking each block
        /// that was allocated for the file as unused if file exists. Otherwise, it returns the Err variant.
        /// Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// let filename1: [u8; 8] = [102, 105, 108, 101, 49, 0, 0, 0]; //file1 as [u8; 8]
        /// let filename2: [u8; 8] = [102, 105, 108, 101, 50, 0, 0, 0]; //file2 as [u8; 8]
        /// my_file_system.create_file(filename1, 8);
        /// my_file_system.create_file(filename2, 4);
        /// my_file_system.delete_file(filename1);
        /// my_file_system.ls();
        /// ```
        ///
        /// This will output the following:
        /// ```text
        /// file2
        /// ```
        pub fn delete_file(&mut self, filename: [u8; 8]) -> Result<(), String> {
            let mut free_block_list = self.get_free_block_list();
            let mut inode = self.get_first_inode_conditional_on(|x| x.name == filename)?;
            for i in 0..inode.size {
                free_block_list[inode.block_pointers[i as usize] as usize] = 0;
            }
            inode.used = 0;
            self.write_inode(inode);
            self.write_free_block_list(free_block_list);
            Ok(())
        }

        /// Prints all current files in the filesystem
        /// Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// my_file_system.ls();
        /// ```
        ///
        /// Will print nothing, as there are no files and just an empty filesystem.
        /// See other doctests for examples where something is printed by ls()
        pub fn ls(&mut self) {
            for i in 0..MAX_INODES {
                let inode = self.get_inode(i);
                if inode.used == 1 {
                    println!("{}", std::str::from_utf8(&inode.name).unwrap());
                }
            }
        }

        /// Reads block block_num out of file and returns Ok(contents) if it exists
        /// and returns Err otherwise.
        /// Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// let filename: [u8; 8] = [102, 105, 108, 101, 49, 0, 0, 0]; //file1 as [u8; 8]
        /// my_file_system.create_file(filename, 8);
        /// println!(my_file_system.read(filename, 7));
        /// ```
        ///
        /// This will output one of the following assuming disk0 exists and the read was successful or not:
        /// ```text
        /// Ok("111....1111")
        /// ```
        /// Or:
        /// ```text
        /// Err("Some error description")
        /// ```
        pub fn read(
            &mut self,
            filename: [u8; 8],
            block_num: u8,
        ) -> Result<[u8; BLOCK_SIZE], String> {
            let inode = self.get_first_inode_conditional_on(|x| x.name == filename)?;
            if inode.size <= block_num {
                return Err(format!(
                    "block_num: {} exceeds capacity of inode.size: {}",
                    block_num, inode.size
                ));
            }
            let block = inode.block_pointers[block_num as usize];
            let mut buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            self.disk
                .seek(std::io::SeekFrom::Start(
                    (FREE_BLOCK_SIZE + BLOCK_SIZE * block as usize) as u64,
                ))
                .unwrap();
            self.disk.read(&mut buf).unwrap();
            return Ok(buf);
        }

        /// Writes to block block_num of file and returns Ok(()) if successful and return Err otherwise.
        /// Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// let filename: [u8; 8] = [102, 105, 108, 101, 49, 0, 0, 0]; //file1 as [u8; 8]
        /// let my_new_data = [69u8; myfs::BLOCK_SIZE];
        /// my_file_system.create_file(filename, 3);
        /// my_filesystem.write(filename, &my_new_data, 2)
        /// println!(my_file_system.read(filename, 2));
        /// ```
        ///
        /// This will output one of the following assuming disk0 exists and the write was successful or not:
        /// ```text
        /// Ok("EEE....EEE")
        /// ```
        /// Or
        /// ```text
        /// Err("Some error description")
        /// ```
        pub fn write(
            &mut self,
            filename: [u8; 8],
            block_num: u8,
            write_buf: &[u8; BLOCK_SIZE],
        ) -> Result<(), String> {
            let inode = self.get_first_inode_conditional_on(|x| x.name == filename)?;
            let block = inode.block_pointers[block_num as usize];
            self.disk
                .seek(std::io::SeekFrom::Start(
                    (FREE_BLOCK_SIZE + BLOCK_SIZE * block as usize) as u64,
                ))
                .unwrap();
            self.disk.write(write_buf).unwrap();
            Ok(())
        }

        // Closes the disk after usage. This is mainly to coincide with the implementation on the C++ side
        // Rusts safety guarentees makes sure that after this function is called on an instance of MyFileSystem,
        // it cannot be referenced again as it takes ownership of self (and then subsequently dropping the owned
        // File inside.
        // Usage:
        /// ```
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// // do some stuff
        /// my_file_system.close_disk();
        /// ```
        ///
        /// And the following will not compile
        /// ```compile_fail
        /// use cs377_filesystem::myfs;
        /// let mut my_file_system = myfs::MyFileSystem::new("disk0");
        /// // do some stuff
        /// my_file_system.close_disk();
        /// my_file_system.ls()
        /// ```
        ///
        /// With the error message
        /// ```text
        /// error[E0382]: borrow of moved value: `my_file_system`
        /// --> src\lib.rs:247:1
        ///     |
        /// 5   | let mut my_file_system = myfs::MyFileSystem::new("disk0");
        ///     |     ------------------ move occurs because `my_file_system` has type `MyFileSystem`, which does not implement the `Copy` trait
        /// 6   | // do some stuff
        /// 7   | my_file_system.close_disk();
        ///     |                ------------ `my_file_system` moved due to this method call
        /// 8   | my_file_system.ls()
        ///     | ^^^^^^^^^^^^^^^^^^^ value borrowed here after move
        ///     |
        /// note: `MyFileSystem::close_disk` takes ownership of the receiver `self`, which moves `my_file_system`
        /// --> C:\Repos\jack-champagne\cs377-final-project\src\lib.rs:254:27
        ///     |
        /// 254 |         pub fn close_disk(self) {
        ///     |                           ^^^^
        /// error: aborting due to previous error
        /// ```
        pub fn close_disk(self) {
            drop(self.disk);
        }
    }

    // This impl block defines private/helper functions for internal implementation
    impl MyFileSystem {
        /// Gets an inode at index in inode table
        fn get_inode(&mut self, inode_index: usize) -> IDXNode {
            let mut inode_buffer = [0u8; IDXNODE_SIZE];
            self.disk
                .seek(std::io::SeekFrom::Start(
                    (FREE_BLOCK_SIZE + IDXNODE_SIZE * inode_index) as u64,
                ))
                .unwrap();
            self.disk.read(&mut inode_buffer).unwrap();
            unsafe { std::mem::transmute::<[u8; IDXNODE_SIZE], IDXNode>(inode_buffer) }
        }

        /// Gets and returns inode conditional on a filter function f
        /// If f returns true for an instance, then the inode is returned. Otherwise an Err
        fn get_first_inode_conditional_on(
            &mut self,
            f: impl Fn(&IDXNode) -> bool,
        ) -> Result<IDXNode, &str> {
            for i in 0..MAX_INODES {
                let inode = self.get_inode(i);
                if f(&inode) {
                    return Ok(inode);
                }
            }
            Err("Could not find inode meeting condition")
        }

        /// This function writes the free_block_list back out to the disk.
        /// This is done after in memory changes to the free_block_list for example when allocating
        /// blocks for a new file.
        fn write_free_block_list(&mut self, free_block_list: [u8; FREE_BLOCK_SIZE]) {
            self.disk.seek(std::io::SeekFrom::Start(0)).unwrap();
            self.disk.write(&free_block_list).unwrap();
        }

        /// This function writes the inode in place over the inode immediately before the current cursor position
        /// It makes the assumption that the cursor position is already placed at the end of the inode position
        /// that overwriting is desired and it does not take in an index as a parameter.
        fn write_inode(&mut self, inode: IDXNode) {
            let inode_buffer = unsafe { std::mem::transmute::<IDXNode, [u8; IDXNODE_SIZE]>(inode) };
            self.disk
                .seek(std::io::SeekFrom::Current(-(IDXNODE_SIZE as i64)))
                .unwrap();
            self.disk.write(&inode_buffer).unwrap();
        }

        /// This returns the free_block_list in byte array format for easy traversal and availability checking.
        fn get_free_block_list(&mut self) -> [u8; FREE_BLOCK_SIZE] {
            self.disk.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut free_block_list: [u8; FREE_BLOCK_SIZE] = [0; FREE_BLOCK_SIZE];
            self.disk.read(&mut free_block_list).unwrap();
            return free_block_list;
        }
    }

    // Written by Jack Champagne
}

#[cfg(test)]
mod tests {
    use crate::myfs::*;
    use std::process::{Command, Stdio};
    fn setup() {
        Command::new("./create_fs")
            .arg("disk0")
            .stdout(Stdio::null())
            .spawn()
            .expect("create_fs failed to run");
    }

    #[test]
    #[should_panic]
    fn bad_fs() {
        setup();
        // Dummy filename in byte format
        let filename = [0, 0, 0, 0, 0, 0, 0, 1];
        let mut my_fs = MyFileSystem::new("diskL");
        my_fs.create_file(filename, 7).unwrap();
    }

    #[test]
    #[should_panic]
    fn bad_file() {
        setup();
        // Dummy filename in byte format
        let filename = [0, 0, 0, 0, 0, 0, 0, 1];
        let mut my_fs = MyFileSystem::new("disk0");
        my_fs.create_file(filename, 100).unwrap();
    }

    #[test]
    fn good_file_ops() {
        setup();
        let mut my_fs = MyFileSystem::new("disk0");
        // 'testfile' in byte format
        let my_filename = [116, 101, 115, 116, 102, 105, 108, 101];
        my_fs.create_file(my_filename, 8).unwrap();
        my_fs.write(my_filename, 1, &[12; BLOCK_SIZE]).unwrap();
        let buf = [0; BLOCK_SIZE];
        assert!(my_fs.read(my_filename, 0).unwrap() == buf);
        assert!(my_fs.read(my_filename, 1).unwrap() != buf);
    }
}
