use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: cargo run reload");
        return;
    }

    if args[0] != "reload" {
        println!("Error: Unknown command");
        return;
    } else {
        println!("Correct command!");
    }
}
