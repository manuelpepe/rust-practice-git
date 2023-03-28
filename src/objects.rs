use core::slice::Iter;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::Digest;
use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct ObjectHeader {
    pub type_: String,
    pub len: usize,
}

impl ObjectHeader {
    pub fn to_string(&self) -> String {
        return format!("{} {}{}", self.type_, self.len, '\0');
    }
}

pub enum GitObject {
    Blob { len: usize, data: String },
    Tree { len: usize, data: String },
}

pub fn objstore_path(sha1digest: &String) -> String {
    return Path::new(".git/objects")
        .join(&sha1digest[..2])
        .join(&sha1digest[2..])
        .to_str()
        .unwrap()
        .to_string();
}

fn get_header(iter: &mut Iter<u8>) -> ObjectHeader {
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
    return ObjectHeader {
        len: size,
        type_: type_.to_string(),
    };
}

fn get_tree_data(iter: &mut Iter<u8>) -> String {
    let mut s = String::new();
    // TODO: Reorder
    // TODO: Add file type to output (calling a func similar to get header, but without reading the whole file)
    //      https://stackoverflow.com/questions/30412521/how-to-read-a-specific-number-of-bytes-from-a-stream
    while iter.len() > 0 {
        // permissions
        // TODO: Leftpad with 0s to 6 chars
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

fn get_blob_data(iter: &mut Iter<u8>) -> String {
    let d: Vec<u8> = iter.map(|x| *x).collect();
    let s = String::from_utf8(d);
    return s.unwrap();
}

pub fn load_object(sha1digest: &String) -> GitObject {
    // Decode file
    let fpath = objstore_path(&sha1digest);
    let file = fs::File::open(fpath).unwrap();
    let mut buf = Vec::new();
    ZlibDecoder::new(file).read_to_end(&mut buf).unwrap();

    // Parse file data
    let mut iter = buf.iter();
    let header = get_header(&mut iter);
    return match header.type_.as_str() {
        "blob" => GitObject::Blob {
            len: header.len,
            data: get_blob_data(&mut iter),
        },
        "tree" => GitObject::Tree {
            len: header.len,
            data: get_tree_data(&mut iter),
        },
        _ => panic!("unkown object type"),
    };
}

pub fn store_object(data: &Vec<u8>, digest: &String, header: ObjectHeader) {
    let pathstr = objstore_path(&digest);
    let outpath = Path::new(&pathstr);
    fs::create_dir_all(outpath.parent().unwrap()).unwrap();
    let mut f = fs::File::create(outpath).unwrap();

    let mut content = Cursor::new(Vec::new());
    content.write(header.to_string().as_bytes()).unwrap();
    content.write(data).unwrap();
    content.seek(SeekFrom::Start(0)).unwrap();

    let mut encoder = ZlibEncoder::new(content, Compression::fast());
    let mut buffer = [0; 1024];
    loop {
        let bytes = encoder.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        f.write(&buffer[..bytes]).unwrap();
    }
}

pub fn calculate_object_hash(header: &ObjectHeader, content: &Vec<u8>) -> String {
    // probably move to objects
    let mut cont = content.clone();
    let mut data = Vec::new();
    data.append(&mut header.to_string().as_bytes().to_vec());
    data.append(&mut cont);

    let mut hash = sha1::Sha1::new();
    hash.update(data);

    let digest = format!("{:x}", hash.finalize());
    return digest;
}
