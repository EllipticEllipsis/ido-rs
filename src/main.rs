use std::fs::read;

mod binasm;
mod debugging;
mod mips;

fn main() -> () {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 2 {
        eprintln!("USAGE: {} FILE", argv[0]);
        return;
    }
    let in_filepath = &argv[1];
    let bytes = read(in_filepath).unwrap();

    println!("Dump of contents of {}", in_filepath);
    debugging::print_bytes(&bytes);

    binasm::process_records(&bytes);
}
