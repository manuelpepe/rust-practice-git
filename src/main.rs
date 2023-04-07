use clap::Parser;
use clap::Subcommand;

use std::fs;
use std::str;

mod clone;
mod files;
mod objects;
mod packs;
mod tree;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// initialize git repository
    Init,

    /// print contents of blob objects
    CatFile {
        object: String,

        #[clap(short = 'p', help = "pretty print object")]
        pretty: bool,
    },

    /// calculate sha1 for file and optionally store the object
    HashObject {
        path: String,

        #[clap(short = 'w', help = "write object to object store")]
        write: bool,
    },

    /// print contents of tree objects
    LsTree {
        treeid: String,

        #[clap(long, help = "print only object names")]
        name_only: bool,
    },

    /// recursively store current working directory as repository objects
    WriteTree,

    /// create commit from a written tree
    CommitTree {
        treeid: String,

        #[clap(short = 'p', help = "parent commit")]
        parent: String,

        #[clap(short = 'm', help = "commit message")]
        message: String,
    },

    /// Clone remote repository
    Clone { url: String, path: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => init(),
        Commands::CatFile { object, pretty: _ } => print!("{}", files::catfile(object)),
        Commands::HashObject { write, path } => println!("{}", files::hashobject(path, *write)),
        Commands::LsTree { treeid, name_only } => {
            let tree = tree::lstree(&treeid);
            for node in tree.iter() {
                if *name_only {
                    println!("{}", node.filename);
                } else {
                    println!("{}\t{}\t{}", node.permissions, node.filename, node.hash);
                }
            }
        }
        Commands::WriteTree => tree::writetree(),
        Commands::CommitTree {
            treeid,
            parent,
            message,
        } => {
            let newcommitid =
                tree::committree(&"manuel@manuel.com".to_string(), treeid, parent, message)
                    .unwrap();
            println!("{}", newcommitid);
        }
        Commands::Clone { url, path } => clone::clone(url, path),
    }
}

fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
