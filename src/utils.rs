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
    println!("args: --dir <directory_folder_path");
}
