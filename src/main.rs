use std::{env, path::PathBuf, process::exit};

mod compression;
mod encryption;
mod file;
mod utils;
// name --dir <path> 


fn main() {
    let args: Vec<String> = env::args().collect();

    if args[2] == "--dir" {
        if !args[3].is_empty() {
            let file_blob = file::read_dir(&PathBuf::from(&args[3])).unwrap();
            let metadata_blob = file::populate_metadata(&file_blob);
            let head = file::create_archive(&metadata_blob);

            //use this for encryption
            let mut final_blob = Vec::new();

            file::append_struct(&mut final_blob, &head);

            for meta in &metadata_blob {
                file::append_struct(&mut final_blob, meta);
            }

            for file in &file_blob {
                final_blob.extend_from_slice(file.name.as_bytes());
            }

            for file in &file_blob {
                final_blob.extend_from_slice(file.path.to_string_lossy().as_bytes());
            }

            for file in &file_blob {
                final_blob.extend_from_slice(&file.content);
            }

            encryption::encrypt_and_compress_flow(&mut final_blob);
            
        }
    } else {
        println!("launch with args\n");
        utils::help();
        exit(1);
    }
}
