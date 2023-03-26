use std::env;
use std::fs;

mod files;
mod objects;
mod tree;

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
    } else if args[1] == "hash-object" {
        let write = args[2] == "-w";
        let path = &args[3];
        println!("{}", files::hashobject(&path, write))
    } else if args[1] == "ls-tree" {
        let treeid: &String;
        let only_name: bool;
        if args[2] == "--name-only" {
            only_name = true;
            treeid = &args[3];
        } else {
            only_name = false;
            treeid = &args[2];
        }
        let tree = tree::lstree(&treeid);
        for node in tree.iter() {
            if only_name {
                println!("{}", node.filename);
            } else {
                println!("{}\t{}\t{}", node.permissions, node.filename, node.hash);
            }
        }
    } else if args[1] == "write-tree" {
        tree::writetree();
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
