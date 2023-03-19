use flate2::read::ZlibDecoder;
use std::fs;
use std::io::prelude::*;

pub fn catfile(blobid: &String) -> String {
    let fpath = format!(".git/objects/{}/{}", &blobid[..2], &blobid[2..]);
    let bdata = fs::read(fpath).unwrap();
    let mut s = String::new();
    ZlibDecoder::new(&bdata[..]).read_to_string(&mut s).unwrap();
    return s;
}
