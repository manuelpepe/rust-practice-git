use bytes::Bytes;

use crate::init;
use crate::objects::{load_object, store_object, GitObjectType};
use crate::packs::{self, ObjectType, Packfile};
use crate::tree::lstree;
use anyhow::{bail, Result};
use std::env::set_current_dir;
use std::fs;
use std::io::Write;
use std::str;

/// Perform a blocking HTTP request to the given URL and download a list of refs
fn discover_refs(url: &String) -> Vec<String> {
    let formatted_url = format!("{}/info/refs?service=git-upload-pack", url);
    let resp = reqwest::blocking::get(formatted_url)
        .unwrap()
        .bytes()
        .unwrap();
    let lines = str::from_utf8(&resp).unwrap().split("\n");
    let iter = lines.skip(2);
    let mut hashes = Vec::new();
    for line in iter {
        if line == "0000" {
            break;
        }
        let h: String = line.chars().skip(4).take(40).collect();
        hashes.push(h);
    }
    return hashes;
}

/// Perform a blocking HTTP request to the given URL and download packfile data
fn request_packfile(url: &String, refs: &Vec<String>) -> Bytes {
    let client = reqwest::blocking::Client::new();
    let formatted_url = format!("{}/git-upload-pack", url);
    let req_body = format!("0032want {}\n00000009done\n", refs.get(0).unwrap());
    let resp = client
        .post(&formatted_url)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(req_body)
        .send()
        .unwrap()
        .bytes()
        .unwrap();
    return resp;
}

/// Store all the objects of the given packfile in the local git object store
fn store_pack_objects(packfile: Packfile) -> Result<()> {
    for entry in packfile.entries {
        match entry.type_ {
            ObjectType::Tree | ObjectType::Blob | ObjectType::Commit => {
                store_object(&entry.type_.to_string(), &entry.data.to_vec())?;
            }
            _ => bail!("storing {} is not supported", entry.type_),
        }
    }
    Ok(())
}

/// Clone a remote repository from the given URL
pub fn clone(url: &String, dest: &String) -> Result<()> {
    let mut base_url = url.clone();
    println!("Cloning '{}' into '{}'", base_url, dest);
    if !base_url.ends_with(".git") {
        base_url.push_str(".git");
    }

    let discovered_refs = discover_refs(&base_url);
    let packfile_data = request_packfile(&base_url, &discovered_refs);
    let packfile = packs::parse_packfile(&packfile_data[8..])?;

    {
        fs::create_dir(&dest)?;
        set_current_dir(&dest)?;

        if let Err(e) = init() {
            set_current_dir("..")?;
            bail!(e);
        };
        if let Err(e) = store_pack_objects(packfile) {
            set_current_dir("..")?;
            bail!(e);
        };

        let head_commit = discovered_refs.get(0).unwrap();
        let checkout_res = checkout_commit(head_commit);
        set_current_dir("..").unwrap();
        if let Err(e) = checkout_res {
            bail!(e);
        }
        return Ok(());
    }
}

/// Creates files and directories in `base` directory
/// to match those of the tree in the given commit.
fn checkout_commit(sha1: &String) -> Result<()> {
    println!("Checking out at {}", sha1);
    let commit = load_object(sha1)?;
    if let GitObjectType::Commit = commit.type_ {
        let head_tree = commit
            .data
            .lines()
            .next()
            .unwrap()
            .split(" ")
            .skip(1)
            .next()
            .unwrap();
        return checkout_tree(&head_tree.to_string(), &String::new());
    } else {
        bail!("head is not a commit object");
    }
}

/// Recursively creates files and directories in `base` directory
/// to match those of the given tree.
fn checkout_tree(sha1: &String, base: &String) -> Result<()> {
    let tree = lstree(sha1)?;
    for node in tree.iter() {
        let mut new_base = format!("{}/{}", base, node.filename);
        new_base = new_base
            .strip_prefix("/")
            .unwrap_or(new_base.as_str())
            .to_string();
        if node.permissions == "40000" {
            fs::create_dir(&new_base)?;
            if let Err(e) = checkout_tree(&node.hash, &new_base) {
                bail!(e);
            };
        } else {
            let blob = load_object(&node.hash)?;
            if let GitObjectType::Blob = blob.type_ {
                let mut f = fs::File::create(new_base)?;
                f.write(blob.data.as_bytes())?;
            } else {
                bail!("treating {} as file", node.hash)
            }
        }
    }
    return Ok(());
}
