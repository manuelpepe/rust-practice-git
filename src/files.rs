use crate::objects;

use std::fs;
use std::io::Read;

pub fn catfile(blobid: &String) -> String {
    let obj = objects::load_object(blobid);
    return match obj.type_ {
        objects::GitObjectType::Blob => obj.data,
        _ => panic!("object not a file"),
    };
}

pub fn hashobject(path: &String, write: bool) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();
    let type_ = &"blob".to_string();
    if write {
        return objects::store_object(type_, &content);
    }
    return objects::calculate_object_hash(type_, &content);
}
