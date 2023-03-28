use clap::Parser;
use clap::Subcommand;

use std::fs;

mod files;
mod objects;
mod tree;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {},
    CatFile {
        object: String,

        #[clap(short = 'p', help = "pretty print object")]
        pretty: bool,
    },
    HashObject {
        path: String,

        #[clap(short = 'w', help = "write object to object store")]
        write: bool,
    },
    LsTree {
        treeid: String,

        #[clap(long, help = "print only object names")]
        only_name: bool,
    },
    WriteTree {},
    CommitTree {
        treeid: String,

        #[clap(short = 'p', help = "parent commit")]
        parent: String,

        #[clap(short = 'm', help = "commit message")]
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init {}) => init(),
        Some(Commands::CatFile { object, pretty: _ }) => {
            print!("{}", files::catfile(object));
        }
        Some(Commands::HashObject { write, path }) => {
            println!("{}", files::hashobject(path, *write))
        }
        Some(Commands::LsTree { treeid, only_name }) => {
            let tree = tree::lstree(&treeid);
            for node in tree.iter() {
                if *only_name {
                    println!("{}", node.filename);
                } else {
                    println!("{}\t{}\t{}", node.permissions, node.filename, node.hash);
                }
            }
        }
        Some(Commands::WriteTree {}) => tree::writetree(),
        Some(Commands::CommitTree {
            treeid,
            parent,
            message,
        }) => {
            let newcommitid =
                tree::committree(&"manuel@manuel.com".to_string(), treeid, parent, message)
                    .unwrap();
            println!("{}", newcommitid);
        }
        None => {
            println!("unknown command")
        }
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
