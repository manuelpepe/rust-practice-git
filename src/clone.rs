use bytes::Bytes;

use crate::init;
use crate::objects::{load_object, store_object, GitObject, ObjectHeader};
use crate::packs::{self, ObjectType, Packfile};
use crate::tree::lstree;
use anyhow::{bail, Ok, Result};
use std::env::set_current_dir;
use std::fs;
use std::io::Write;
use std::str;

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

fn store_pack_objects(packfile: Packfile) {
    for entry in packfile.entries {
        match entry.type_ {
            ObjectType::Tree | ObjectType::Blob | ObjectType::Commit => store_object(
                &entry.data.to_vec(),
                &entry.sha1,
                ObjectHeader {
                    type_: entry.type_.to_string(),
                    len: entry.size,
                },
            ),
            _ => {
                panic!("storing {} is not supported", entry.type_);
            }
        }
    }
}

pub fn clone(url: &String, dest: &String) {
    let mut base_url = url.clone();
    println!("Cloning '{}' into '{}'", base_url, dest);
    if !base_url.ends_with(".git") {
        base_url.push_str(".git");
    }

    let discovered_refs = discover_refs(&base_url);
    let packfile_data = request_packfile(&base_url, &discovered_refs);
    let packfile = packs::parse_packfile(&packfile_data[8..]);

    {
        fs::create_dir(&dest).unwrap();
        set_current_dir(&dest).unwrap();
        init();
        store_pack_objects(packfile);
        let head_commit = discovered_refs.get(0).unwrap();
        let checkout_res = checkout_commit(head_commit, &String::new());
        set_current_dir("..").unwrap();
        if let Err(e) = checkout_res {
            panic!("{}", e)
        }
    }
}

fn checkout_commit(sha1: &String, base: &String) -> Result<()> {
    println!("Checking out at {}", sha1);
    let commit = load_object(sha1);
    if let GitObject::Commit { len: _, data } = commit {
        let head_tree = data
            .lines()
            .next()
            .unwrap()
            .split(" ")
            .skip(1)
            .next()
            .unwrap();
        return checkout_tree(&head_tree.to_string(), &base);
    } else {
        bail!("head is not a commit object");
    }
}

/// Checkout tree to base directory.
fn checkout_tree(sha1: &String, base: &String) -> Result<()> {
    let tree = lstree(sha1);
    for node in tree.iter() {
        let mut new_base = format!("{}/{}", base, node.filename);
        new_base = new_base
            .strip_prefix("/")
            .unwrap_or(new_base.as_str())
            .to_string();
        if node.permissions == "40000" {
            fs::create_dir(&new_base).unwrap();
            if let Err(e) = checkout_tree(&node.hash, &new_base) {
                bail!(e);
            };
        } else {
            if let GitObject::Blob { len: _, data } = load_object(&node.hash) {
                let mut f = fs::File::create(new_base).unwrap();
                f.write(data.as_bytes()).unwrap();
            } else {
                bail!("treating {} as file", node.hash)
            }
        }
    }
    return Ok(());
}
