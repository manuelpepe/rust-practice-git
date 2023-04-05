use crate::objects::{store_object, ObjectHeader};
use crate::packs::{self, ObjectType, Packfile};
use std::str;

fn get_discovered(data: &[u8]) -> Vec<String> {
    let lines = str::from_utf8(&data).unwrap().split("\n");
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

pub fn clone(url: &String) {
    let mut base_url = url.clone();
    println!("Cloning '{}'", base_url);
    if !base_url.ends_with(".git") {
        base_url.push_str(".git");
    }

    let first_url = format!("{}/info/refs?service=git-upload-pack", base_url.clone());
    let resp = reqwest::blocking::get(first_url).unwrap().bytes().unwrap();
    let discovered_refs = get_discovered(&resp);

    let client = reqwest::blocking::Client::new();
    let second_url = format!("{}/git-upload-pack", base_url.clone());
    let req_body = format!(
        "0032want {}\n00000009done\n",
        discovered_refs.get(0).unwrap()
    );
    let resp = client
        .post(&second_url)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(req_body)
        .send()
        .unwrap()
        .bytes()
        .unwrap();

    let packfile = packs::parse_packfile(&resp[8..]);
    store_pack_objects(packfile);
    checkout_ref(discovered_refs.get(0).unwrap())
}

fn checkout_ref(sha1: &String) {}
