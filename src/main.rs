use std::env::args_os;

mod myfs;

fn main() {
    if args_os().len() == 1 {
        eprintln!("usage: {:?} <filename>\n", args_os());
    }

    let my_file_system = myfs::MyFileSystem::new(String::from("disk0"));
}
