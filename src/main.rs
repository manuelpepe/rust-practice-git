use anyhow::Result;
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
        Commands::Init => {
            init().unwrap();
        }
        Commands::CatFile { object, pretty: _ } => {
            print!("{}", files::catfile(object).unwrap());
        }
        Commands::HashObject { write, path } => {
            println!("{}", files::hashobject(path, *write).unwrap())
        }
        Commands::LsTree { treeid, name_only } => {
            let tree = tree::lstree(&treeid).unwrap();
            for node in tree.iter() {
                if *name_only {
                    println!("{}", node.filename);
                } else {
                    println!("{}\t{}\t{}", node.permissions, node.filename, node.hash);
                }
            }
        }
        Commands::WriteTree => {
            let digest = tree::writetree().unwrap();
            println!("{}", digest);
        }
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
        Commands::Clone { url, path } => {
            clone::clone(url, path).unwrap();
        }
    }
}

fn init() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    println!("Initialized git directory");
    return Ok(());
}

#[cfg(test)]
mod testutils {
    use std::env::{current_dir, set_current_dir};
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Mutex;
    use tempdir::TempDir;

    use crate::init;

    static MUTEX: Mutex<u8> = Mutex::new(0u8);

    pub fn get_git_output(args: &[&str]) -> String {
        let stdout = Command::new("git")
            .args(args)
            .output()
            .expect("error running git")
            .stdout;
        return String::from_utf8(stdout).unwrap();
    }

    fn tempdir() -> (PathBuf, TempDir) {
        let cwd = current_dir().unwrap();
        let tempdir = TempDir::new("gittest").unwrap();
        assert!(tempdir.path().is_dir());
        return (cwd, tempdir);
    }

    pub fn in_tmp_dir<F>(func: F)
    where
        F: FnOnce() -> (),
    {
        let _lock = match MUTEX.lock() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        let (cwd, dir) = tempdir();
        set_current_dir(&dir).unwrap();
        func();
        set_current_dir(cwd).unwrap();
    }

    pub fn in_tmp_git<F>(func: F)
    where
        F: FnOnce() -> (),
    {
        in_tmp_dir(|| {
            init().unwrap();
            func();
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils;
    use std::path;

    #[test]
    fn test_init() {
        testutils::in_tmp_dir(|| {
            crate::init().unwrap();
            assert!(path::Path::new(".git").exists());
            let data = testutils::get_git_output(&["status"]);
            assert!(data.contains("On branch master"));
            assert!(data.contains("No commits yet"));
            assert!(data.contains("nothing to commit"));
        });
    }
}
