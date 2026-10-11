use std::io::{self, Write};
use std::process;
use std::time::Duration;

use crate::{fork_patterns, more};

fn read_line(prompt: &str) -> String {
    println!("{}", prompt);
    io::stdout().flush().unwrap(); // clean everting

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("Falure to read line.");
    buf.trim().to_string()
}

fn read_num(prompt: &str) -> Option<u64> {
    loop {
        let input = read_line(prompt);
        if input.eq_ignore_ascii_case("exit") {
            return None;
        }
        match input.parse::<u64>() {
            Ok(n) => return Some(n),
            Err(_) => println!("Not a valid number. try again or type 'exit'."),
        }
    }
}

fn print_menu() {
    println!("\n===============| TOPICS |===============");
    for (i, (name, _)) in TOPICS.iter().enumerate() {
        println!(" {}) {}", i + 1, name);
    }
    println!(" 0) to Quit, or type 'exit'");
}

pub fn print_loop(orignal_pid: u32) {
    loop {
        print_menu();
        let choice = match read_num("Choose Topic:") {
            Some(n) => n,
            None => {
                println!("Bye!");
                return;
            }
        };

        match choice {
            0 => {
                println!("Bye!");
                return;
            }
            n if (n as usize) <= TOPICS.len() && n > 0 => {
                let (_, func) = TOPICS[n as usize - 1];
                println!();
                func();
                if process::id() != orignal_pid {
                    process::exit(0);
                }
                println!("💟🩷💚💗💛❤️💙💜");
                std::thread::sleep(Duration::from_secs(3));
            }
            _ => println!("No Such Topic."),
        }
    }
}

const TOPICS: &[(&str, fn())] = &[
    //
    ("Fork execution", fork_patterns::fork_),
    ("Fork with Wait", fork_patterns::double_fork_n_wait),
    ("fork_", fork_patterns::fork_),
    ("fork_n_wait", fork_patterns::double_fork_n_wait),
    ("fork_n_execvp", fork_patterns::fork_n_execvp),
    ("fork_exec_open_close", fork_patterns::fork_exec_open_close),
    ("read_write", more::read_write),
    ("duplicate_fd", more::duplicate_fd),
    ("duplicate_two", more::duplicate_two),
    ("redirect_stdout", more::redirect_stdout),
    ("redirect_stdout_x", more::redirect_stdout_x),
    ("simple_pipe", more::simple_pipe),
    ("pipe_communication", more::pipe_communication),
];
