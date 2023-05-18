# cs377-final-project
File system project in Rust
Author: Jack Champagne

## Description
This project aims to implement as closely as possible project 6 (File systems) in the Rust programming language.

Things that will be directly translated:
- [X] - main.cpp
- [X] - fs.cpp
- [X] - create_fs.cpp

## Usage 
After cloning
```bash
cargo build
```
The executables are 'create_fs' and 'fs_app' inside of the targets/debug or targets/release subdirectories.

Run them like so if they are in targets/debug for example
```bash
cargo run --bin create_fs disk0
```

```bash
cargo run --bin fs_app .\sample.txt
```

## Documentation
For complete project documentation, please visit the following link:

**[rustdoc docs on github pages](https://jack-champagne.github.io/cs377-final-project/cs377_filesystem/index.html)**

This is inside of the docs folder of this repo and was generated using `cargo docs`

Thank you!
