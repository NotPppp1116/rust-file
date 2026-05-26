use rand::Rng;
use rand::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, path::PathBuf, process::exit};

mod compression;
mod decrypt;
mod encryption;
mod file;
mod read_back;
mod send;
mod utils;
// name --dir <path>

const NAME: &str = "mole";
const EXT: &str = "bin";
//TODO make this better to a out file of argument
macro_rules! uinique_name {
    () => {{
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();

        let mut rng = rand::rng();
        let num: u64 = rng.random();
        format!("{}_{}_{}_{}.{}", NAME, num, pid, now, EXT)
    }};
}
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command_index = match args.get(1).map(String::as_str) {
        Some("--enc" | "--dec" | "--mole") => 1,
        _ => 2,
    };

    let file_name = uinique_name!();

    match args.get(command_index).map(String::as_str) {
        Some("--enc") if args.get(command_index + 1).map(String::as_str) == Some("--dir") => {
            let Some(directory_path) = args.get(command_index + 2).filter(|path| !path.is_empty())
            else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };

            let file_blob = file::read_dir(&PathBuf::from(directory_path)).unwrap();
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

            let finale = encryption::encrypt_and_compress_flow(&mut final_blob);

            //check if we want to send somewhere or no
            match (
                args.get(command_index + 3).map(String::as_str),
                args.get(command_index + 4),
            ) {
                (Some("--send"), Some(destination)) if !destination.is_empty() => {
                    send::send_single(destination, &finale)
                        .await
                        .expect("failed to send archive");
                }
                (Some("--recieve"), Some(port)) if !port.is_empty() => {
                    send::recieve_single(port)
                        .await
                        .expect("failed to receive archive");
                }
                _ => {}
            }
            fs::write(file_name, &finale).expect("failed to write archive");

            fs::remove_dir_all(directory_path).expect("failed to delete file");
        }
        Some("--dec") => {
            let Some(archive_path) = args.get(command_index + 1).filter(|path| !path.is_empty())
            else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };
            let archive = decrypt::decrypt_and_decomp(archive_path);
            let output_root = args.get(command_index + 2).map_or(".", String::as_str);

            read_back::restore_archive(&archive, output_root).expect("failed to restore archive");
        }
        Some("--mole") => utils::easteregg(),
        _ => {
            println!("launch with args\n");
            utils::help();
            exit(1);
        }
    }
}
