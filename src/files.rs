use crate::objects;

use anyhow::{bail, Result};
use std::fs;
use std::io::Read;

pub fn catfile(blobid: &String) -> Result<String> {
    let obj = objects::load_object(blobid)?;
    return match obj.type_ {
        objects::GitObjectType::Blob => Ok(obj.data),
        _ => bail!("object not a file"),
    };
}

pub fn hashobject(path: &String, write: bool) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let type_ = &"blob".to_string();
    if write {
        return objects::store_object(type_, &content);
    }
    return Ok(objects::calculate_object_hash(type_, &content));
}

}
