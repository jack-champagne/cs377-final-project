use std::{env::args_os, fs::File, io::{Read, BufReader, BufRead}};

mod myfs;

const BUFFER_SIZE: usize = myfs::BLOCK_SIZE;

fn main() {
    let mut args: Vec<_> = args_os().collect();
    if args.len() == 1 {
        eprintln!("usage: {:?} <filename>\n", args_os());
        return;
    }

    let my_file_system = myfs::MyFileSystem::new("disk0");
    let buff: [u8; BUFFER_SIZE] = [b'1'; BUFFER_SIZE];
    let mut instruction_file = File::open(args[1].as_os_str()).unwrap();
    let reader = BufReader::new(instruction_file);

    for result in reader.lines() {
        let line_string =  result.unwrap();
        do_file_op(&line_string.as_bytes().to_vec())
    }
}

fn do_file_op(line: &Vec<u8>) {
    match &line[0] {
        b'C' => todo!(),
        b'D' => todo!(),
        op => {
            let char = *op as char;
            println!("File operation {char:?} is unimplemented")
        },
    }
}
