use std::env;
use std::fs;

mod files;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        init()
    } else if args[1] == "cat-file" {
        if args[2] == "-p" {
            let blobid: &String = &args[3];
            let data = files::catfile(&blobid);
            print!("{}", data);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
