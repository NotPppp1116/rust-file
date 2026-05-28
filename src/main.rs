use std::{env, fs, path::PathBuf, process::exit};

mod compression;
mod debug_safety;
mod decrypt;
mod discovery;
mod encryption;
mod file;
mod google_drive;
mod read_back;
mod send;
mod utils;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command_index = match args.get(1).map(String::as_str) {
        Some("encrypt" | "decrypt" | "mole" | "receive" | "serve-discovery" | "find-receiver") => 1,
        _ => 2,
    };

    let file_name = utils::generate_uuid_name();

    match args.get(command_index).map(String::as_str) {
        Some("encrypt") if args.get(command_index + 1).map(String::as_str) == Some("--dir") => {
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

            file::append_archive_head(&mut final_blob, &head);

            for meta in &metadata_blob {
                file::append_metadata(&mut final_blob, meta);
            }

            for file in &file_blob {
                final_blob.extend_from_slice(file.name.as_bytes());
            }

            for file in &file_blob {
                final_blob.extend_from_slice(&file::path_archive_bytes(&file.path));
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
                _ => {}
            }
            fs::write(file_name, &finale).expect("failed to write archive");

            fs::remove_dir_all(directory_path).expect("failed to delete file");
        }
        Some("receive") => {
            let Some(port) = args.get(command_index + 1).filter(|port| !port.is_empty()) else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };

            let received = send::receive_single(port)
                .await
                .expect("failed to receive archive");
            fs::write(file_name, received).expect("failed to write received archive");
        }
        Some("decrypt") => {
            let Some(archive_path) = args.get(command_index + 1).filter(|path| !path.is_empty())
            else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };
            let archive =
                decrypt::decrypt_and_decomp(archive_path).expect("failed to decrypt archive");
            let output_root = args.get(command_index + 2).map_or(".", String::as_str);

            read_back::restore_archive(&archive, output_root).expect("failed to restore archive");
        }
        Some("mole") => utils::easteregg(),
        Some("serve-discovery") => {
            let (Some(file_name), Some(receiver_addr)) =
                (args.get(command_index + 1), args.get(command_index + 2))
            else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };

            discovery::discovery_serve(file_name, receiver_addr)
                .await
                .expect("failed to serve receiver discovery");
        }
        Some("find-receiver") => {
            let Some(file_name) = args.get(command_index + 1) else {
                println!("launch with args\n");
                utils::help();
                exit(1);
            };

            let receiver = discovery::find_receiver(file_name)
                .await
                .expect("failed to find receiver");
            println!("{receiver}");
        }
        _ => {
            println!("launch with args\n");
            utils::help();
            exit(1);
        }
    }
}
