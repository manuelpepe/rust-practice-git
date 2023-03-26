use crate::objects;

use std::fs;
use std::io::prelude::*;

pub fn catfile(blobid: &String) -> String {
    // FIXME: Fail if blobid.len() != 40
    // FIXME: Use path joining instead of string formatting
    let obj = objects::load_object(blobid);
    return match obj {
        objects::GitObject::Blob { data, .. } => data,
        _ => panic!("object not a file"),
    };
}

pub fn hashobject(path: &String, write: bool) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut content = Vec::new();
    let bytes_read = file.read_to_end(&mut content).unwrap();
    let header = objects::ObjectHeader {
        type_: "blob".to_string(),
        len: bytes_read,
    };
    let digest = objects::calculate_object_hash(&header, &content);
    if write {
        objects::store_object(&content, &digest, header);
    }
    return digest;
}
