use crate::objects;

use sha1::Digest;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;

pub fn catfile(blobid: &String) -> String {
    // FIXME: Fail if blobid.len() != 40
    // FIXME: Use path joining instead of string formatting
    let obj = objects::load_object(blobid);
    return match obj {
        objects::GitObject::Blob { data, .. } => data,
        _ => panic!("object not a file"),
    };
}

fn calculate_object_hash(header: objects::ObjectHeader, content: &mut String) -> String {
    // probably move to objects
    let mut data = String::new();
    data.push_str(&header.to_string());
    data.push_str(&content);

    let mut hash = sha1::Sha1::new();
    hash.update(data);

    let digest = format!("{:x}", hash.finalize());
    return digest;
}

pub fn hashobject(path: &String, write: bool) -> String {
    let mut content = String::new();
    let mut file = fs::File::open(path).unwrap();
    let bytes_read = file.read_to_string(&mut content).unwrap();
    let header = objects::ObjectHeader {
        type_: "blob".to_string(),
        len: bytes_read,
    };
    let digest = calculate_object_hash(header, &mut content);
    if write {
        file.seek(SeekFrom::Start(0)).unwrap();
        objects::store_object(
            &mut content,
            &digest,
            objects::ObjectHeader {
                type_: "blob".to_string(),
                len: bytes_read,
            },
        );
    }
    return digest;
}
