use helper_functions::print_loop;
use std::process;

mod fork_patterns;
mod helper_functions;
mod more;

fn main() {
    let orignal_pid = process::id();
    println!("MAIN_PID:{}", orignal_pid);
    print_loop(orignal_pid);
}
