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

#[cfg(test)]
mod tests {
    use crate::files::catfile;
    use crate::objects::objstore_path;
    use crate::testutils;
    use std::fs;
    use std::path;

    use super::hashobject;

    #[test]
    fn test_hashobject_and_catfile() {
        testutils::in_tmp_git(|| {
            let covs = vec![
                ("myfile.txt", "my contents\n"),
                ("another_file.py", "another\nfile\ncontents\n"),
            ];
            for (path, content) in covs {
                fs::write(path, content).expect("should be able to write file");

                let sha1 = hashobject(&String::from(path), false).unwrap();
                assert!(!path::Path::new(&objstore_path(&sha1)).exists());

                let sha1_2 = hashobject(&String::from(path), true).unwrap();
                assert_eq!(sha1, sha1_2);
                assert!(path::Path::new(&objstore_path(&sha1)).exists());

                let read_content = catfile(&sha1).unwrap();
                assert!(read_content.eq(content));

                let git_read_content =
                    testutils::get_git_output(&["cat-file", "-p", &sha1.as_str()]);
                assert_eq!(git_read_content, read_content);
            }
        });
    }
}
