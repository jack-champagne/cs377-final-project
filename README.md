# cs377-final-project
File system project in Rust
Author: Jack Champagne

## Description
This project aims to implement as closely as possible project 6 (File systems) in the Rust programming language.

Things that will be directly translated:
[X] - main.cpp
[X] - fs.cpp
[X] - create_fs.cpp

## Usage 
After cloning
```bash
cargo build
```
The executables are 'create_fs' and 'fs_app' inside of the targets/debug or targets/release subdirectories.

Run them like so if they are in targets/debug for example
```bash
./targets/debug/create_fs disk0
```

```bash
./targets/debug/fs_app sample.txt
```