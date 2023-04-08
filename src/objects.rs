use anyhow::{bail, Result};
use core::slice::Iter;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::Digest;
use std::fmt::Display;
use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// GitObject is a public facing struct representing a `loaded` git object.
#[derive(Debug)]
pub struct GitObject {
    pub type_: GitObjectType,
    pub data: String,
}

#[derive(Debug)]
pub enum GitObjectType {
    Blob,
    Tree,
    Commit,
}

impl Display for GitObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_ = match self {
            GitObjectType::Commit => "commit",
            GitObjectType::Tree => "tree",
            GitObjectType::Blob => "blob",
        };
        return write!(f, "{}", type_);
    }
}

impl GitObjectType {
    fn from_string(string: &str) -> GitObjectType {
        match string {
            "commit" => GitObjectType::Commit,
            "tree" => GitObjectType::Tree,
            "blob" => GitObjectType::Blob,
            _ => panic!("git object type not known"),
        }
    }
}

/// Returns the relative path to a git object given its hash, in the cwd
pub fn objstore_path(sha1digest: &String) -> String {
    return Path::new(".git/objects")
        .join(&sha1digest[..2])
        .join(&sha1digest[2..])
        .to_str()
        .unwrap()
        .to_string();
}

/// Returns size and type of git object from iterator of binary data
fn parse_header(iter: &mut Iter<u8>) -> (usize, GitObjectType) {
    let mut buf = String::new();
    for &i in iter {
        if i == 0 {
            break;
        }
        buf.push(i as char)
    }
    let mut header_parts = buf.split(" ");
    let type_ = header_parts.next().unwrap();
    let size: usize = header_parts.next().unwrap().parse().unwrap();
    return (size, GitObjectType::from_string(type_));
}

/// Parses tree data as String from iterator of binary data
fn parse_tree_data(iter: &mut Iter<u8>) -> String {
    let mut s = String::new();
    // TODO: Add file type to output (calling a func similar to get header, but without reading the whole file)
    //      https://stackoverflow.com/questions/30412521/how-to-read-a-specific-number-of-bytes-from-a-stream
    while iter.len() > 0 {
        // permissions
        for &i in iter.by_ref() {
            if i == 0x20 {
                break;
            }
            s.push(i as char);
        }
        s.push('\t');

        // filenames
        for &i in iter.by_ref() {
            if i == 0 {
                break;
            }
            s.push(i as char);
        }
        s.push('\t');

        // hashes
        let mut buffer = Vec::new();
        for _ in 0..20 {
            let i = iter.next().unwrap();
            buffer.push(*i);
        }
        s.push_str(hex::encode(&buffer).as_str());
        s.push('\n');
    }

    return s;
}

/// Parses blob data as String from iterator of binary data
fn parse_blob_data(iter: &mut Iter<u8>) -> String {
    let d: Vec<u8> = iter.map(|x| *x).collect();
    let s = String::from_utf8(d);
    return s.unwrap();
}

/// Loads object from local git object store and returns a GitObject
pub fn load_object(sha1digest: &String) -> Result<GitObject> {
    // Decode file
    let fpath = objstore_path(&sha1digest);
    let file = match fs::File::open(&fpath) {
        Ok(f) => f,
        Err(_) => bail!("file '{}' does not exists", &fpath),
    };

    let mut buf = Vec::new();
    if let Err(e) = ZlibDecoder::new(file).read_to_end(&mut buf) {
        bail!(e)
    };

    // Parse file data
    let mut iter = buf.iter();
    let (_, type_) = parse_header(&mut iter);
    let data = match type_ {
        GitObjectType::Blob | GitObjectType::Commit => parse_blob_data(&mut iter),
        GitObjectType::Tree => parse_tree_data(&mut iter),
    };
    return Ok(GitObject {
        type_: type_,
        data: data,
    });
}

/// Prepares object data for hashing and writting
fn prepare_data(type_: &String, data: &Vec<u8>) -> Cursor<Vec<u8>> {
    let mut content = Cursor::new(Vec::new());
    content
        .write(
            format!("{} {}{}", type_, data.len(), '\0')
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    content.write(data).unwrap();
    content.seek(SeekFrom::Start(0)).unwrap();
    return content;
}

/// Writes ZlibEncoder contents into a file
fn write_encoder<R: Read>(encoder: &mut ZlibEncoder<R>, file: &mut fs::File) -> Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let bytes = encoder.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        file.write(&buffer[..bytes])?;
    }
    Ok(())
}

/// Stores object in local git object database (in cwd)
pub fn store_object(type_: &String, data: &Vec<u8>) -> Result<String> {
    let mut data_to_write = prepare_data(type_, data);
    let sha1 = inner_calculate_object_hash(&mut data_to_write);
    data_to_write.seek(SeekFrom::Start(0))?;

    let pathstr = objstore_path(&sha1);
    let outpath = Path::new(&pathstr);
    fs::create_dir_all(outpath.parent().unwrap())?;

    let mut file = fs::File::create(outpath)?;
    let mut encoder = ZlibEncoder::new(data_to_write, Compression::fast());
    write_encoder(&mut encoder, &mut file)?;
    return Ok(sha1);
}

/// Calculates object sha1 hash
fn inner_calculate_object_hash(data: &mut Cursor<Vec<u8>>) -> String {
    let mut hash = sha1::Sha1::new();
    hash.update(data.get_mut());
    return format!("{:x}", hash.finalize());
}

/// Calculates object sha1 hash
pub fn calculate_object_hash(type_: &String, data: &Vec<u8>) -> String {
    let mut cursor = prepare_data(type_, data);
    return inner_calculate_object_hash(&mut cursor);
}
