use std::{
    env::args_os,
    fs::File,
    io::{BufRead, BufReader},
};

mod myfs;

const BUFFER_SIZE: usize = myfs::BLOCK_SIZE;
const BUFF: [u8; BUFFER_SIZE] = [b'1'; BUFFER_SIZE];

fn main() {
    let args: Vec<_> = args_os().collect();
    if args.len() == 1 {
        eprintln!("usage: {:?} <filename>\n", args_os());
        return;
    }

    let mut my_file_system = myfs::MyFileSystem::new("disk0");
    let instruction_file = File::open(args[1].as_os_str()).unwrap();
    let reader = BufReader::new(instruction_file);

    for result in reader.lines() {
        let mut line_string = result.unwrap();
        do_file_op(&mut my_file_system, &mut line_string)
    }
    my_file_system.close_disk();
}

pub fn get_filename_array(filename: &str) -> [u8; 8] {
    let mut filename_array: [u8; 8] = [0; 8];
    let bytes = filename.as_bytes();
    for i in 0..bytes.len().min(8) {
        filename_array[i] = bytes[i];
    }
    filename_array
}

fn do_file_op(my_fs: &mut myfs::MyFileSystem, line: &mut String) {
    let mut split_parts = line.split_ascii_whitespace();
    let op = split_parts.next().unwrap();
    let args: Vec<&str> = split_parts.collect();
    match &op.chars().next().unwrap() {
        'C' => {
            let filename = get_filename_array(args[0]);
            let size = args[1].parse().unwrap();
            my_fs
                .create_file(filename, size)
                .expect("Creation of file failed");
        }
        'W' => {
            let filename = get_filename_array(args[0]);
            let block_num = args[1].parse().unwrap();
            my_fs.write(filename, block_num, BUFF);
        }
        'L' => {
            my_fs.ls();
        }
        'R' => {
            let filename = get_filename_array(args[0]);
            let block_num = args[1].parse().unwrap();
            match my_fs.read(filename, block_num) {
                Some(block) => println!("{:?}", std::str::from_utf8(&block)),
                None => todo!(),
            };
        }
        'D' => {
            let filename = get_filename_array(args[0]);
            my_fs.delete_file(filename);
        }
        _ => (),
    }
}

// Written by Jack Champagne
