use std::io::{self, Write};

pub fn ask_number(message: &str) -> i32 {
    let mut content = String::new();
    print!("{message}");
    io::stdout().flush().expect("failed to flush stdout");
    io::stdin()
        .read_line(&mut content)
        .expect("failed to read input");
    content.trim().parse().unwrap()
}
pub fn help() {
    println!("usage:");
    println!("  rust-file <ignored> --enc --dir <directory_path>");
    println!("  rust-file <ignored> --dec <archive_path> [output_directory]");
}
