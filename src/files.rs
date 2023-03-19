use flate2::read::ZlibDecoder;
use std::fs;
use std::io::prelude::*;

pub fn catfile(blobid: &String) -> String {
    let fpath = format!(".git/objects/{}/{}", &blobid[..2], &blobid[2..]);
    let file = fs::File::open(fpath).unwrap();
    let mut s = String::new();
    ZlibDecoder::new(file).read_to_string(&mut s).unwrap();
    return s;
}
