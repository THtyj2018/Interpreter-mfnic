use std::io::{stdout, Write};

use mfni::*;

fn main() {
    let mut it = Interpreter::new();
    let mut prefix = ">>> ";
    loop {
        print!("{}", prefix);
        stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();
        if prefix == ">>> " && line == "quit" {
            break;
        }
        let line = std::ffi::CString::new(line).unwrap();
        match it.input(line.as_bytes_with_nul()) {
            Ok(state) => match state {
                InputState::Empty => (),
                InputState::Incomplete => prefix = "... ",
                InputState::Assignment => prefix = ">>> ",
                InputState::Expression => {
                    println!("{}", it.last_result());
                    prefix = ">>> ";
                }
            },
            Err(e) => {
                eprintln!("!Error: {}", e.to_string());
                prefix = ">>> ";
            }
        }
    }
}
