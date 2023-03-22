use flate2::read::ZlibDecoder;
use flate2::read::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

fn objstore_path(sha1digest: &String) -> String {
    return Path::new(".git/objects")
        .join(&sha1digest[..2])
        .join(&sha1digest[2..])
        .to_str()
        .unwrap()
        .to_string();
}

pub fn catfile(blobid: &String) -> String {
    // FIXME: Fail if blobid.len() != 40
    // FIXME: Use path joining instead of string formatting
    let fpath = objstore_path(&blobid);
    let file = fs::File::open(fpath).unwrap();
    let mut s = String::new();
    ZlibDecoder::new(file).read_to_string(&mut s).unwrap();
    return s[s.find('\0').unwrap() + 1..].to_string();
}

fn calculate_object_hash(file: &mut File) -> (String, usize) {
    let mut content = String::new();
    let bytes_read = file.read_to_string(&mut content).unwrap();

    let mut header = format!("blob {}{}", bytes_read, '\0');
    header.push_str(&content);
    let mut hash = sha1::Sha1::new();
    hash.update(header);

    let digest = format!("{:x}", hash.finalize());
    return (digest, bytes_read);
}

fn write_object_to_store(file: &mut File, digest: &String, len: usize) {
    let pathstr = objstore_path(&digest);
    let outpath = Path::new(&pathstr);
    fs::create_dir_all(outpath.parent().unwrap()).unwrap();
    let mut f = File::create(outpath).unwrap();

    let mut content = format!("blob {}{}", len, '\0');
    file.read_to_string(&mut content).unwrap();

    let mut encoder = ZlibEncoder::new(content.as_bytes(), Compression::fast());
    let mut buffer = [0; 1024];
    loop {
        let bytes = encoder.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        f.write(&buffer).unwrap();
    }
}

pub fn hashobject(path: &String, write: bool) -> String {
    let mut file = fs::File::open(path).unwrap();
    let (digest, len) = calculate_object_hash(&mut file);
    if write {
        file.seek(SeekFrom::Start(0)).unwrap();
        write_object_to_store(&mut file, &digest, len)
    }
    return digest;
}
