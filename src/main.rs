use std::{env, fs, path::PathBuf, process::exit};

mod compression;
mod decrypt;
mod encryption;
mod file;
mod utils;
// name --dir <path>

fn main() {
    let args: Vec<String> = env::args().collect();

    if args[2] == "--enc" && !args[3].is_empty() {
        if args[3] == "--dir" && !args[4].is_empty() {
            let file_blob = file::read_dir(&PathBuf::from(&args[4])).unwrap();
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

            //only keep the encrypted and compressed
            fs::remove_file(&args[4]).expect("failed to delete file");
        }
    } else if args[2] == "--dec" && !args[3].is_empty() {
        let _archive = decrypt::decrypt_and_decomp(&args[3]);

        //TODO: now read the contents make the structs and reconstruct the original things
    } else {
        println!("launch with args\n");
        utils::help();
        exit(1);
    }
}
