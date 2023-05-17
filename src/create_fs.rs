
use std::{env::args_os, fs::File, io::Write};

// create a file  to act as a disk  and format the file system residing on the disk

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = args_os().collect();
    if args.len() == 1 {
        eprintln!("usage: {:?} <diskFileName> ", args[0].as_os_str());
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
    println!("Creating a 128KB file in {:?}", args[1]);
    println!("This file will act as a dummy disk and will hold your filesystem");

    let mut my_disk = File::create(&args[1])?;

    println!("Formatting your filesystem...");

    let mut buf = [0u8; 1024];
    // Mark superblock as allocated in the free block list all other blocks
    // are free, all inodes are zeroed out.
    buf[0] = 1;

    // Write out the superblock
    my_disk.write(&buf)?;

    buf[0] = 0;

    for _ in 0..127 {
        my_disk.write(&buf)?;
    }

    Ok(())
}