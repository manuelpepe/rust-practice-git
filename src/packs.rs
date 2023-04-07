use std::{
    collections::HashMap,
    fmt::Display,
    io::{Cursor, Read},
};

use bytes::{Buf, Bytes};
use flate2::bufread::ZlibDecoder;

use crate::objects::calculate_object_hash;

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 5,
    RefDelta = 6,
}

impl ObjectType {
    fn from_u8(b: u8) -> Self {
        match b {
            1 => ObjectType::Commit,
            2 => ObjectType::Tree,
            3 => ObjectType::Blob,
            4 => ObjectType::Tag,
            6 => ObjectType::OfsDelta,
            7 => ObjectType::RefDelta,
            _ => panic!("unexpected object type {b:08b}"),
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_ = match self {
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Blob => "blob",
            ObjectType::Tag => "tag",
            ObjectType::OfsDelta => "ofs-delta",
            ObjectType::RefDelta => "ref-delta",
        };
        return write!(f, "{}", type_);
    }
}

#[derive(Debug)]
pub struct Packfile {
    pub sha1: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub type_: ObjectType,
    pub size: usize,
    pub sha1: String,
    pub data: Bytes,
}

fn unpack_compressed_data(data: &[u8]) -> Result<(usize, Bytes), ()> {
    let bytes = Bytes::from(data.to_vec());
    let mut decoder = ZlibDecoder::new(bytes.as_ref());
    let mut content = Vec::new();
    let r = decoder.read_to_end(&mut content);
    match r {
        Ok(_size) => return Ok((decoder.total_in() as usize, Bytes::from(content))),
        Err(_err) => return Ok((0, Bytes::new())),
    };
}

fn parse_header(data: &[u8]) -> u32 {
    let pack = String::from_utf8(data[..4].try_into().unwrap()).unwrap();
    assert_eq!(pack, "PACK");
    let version = u32::from_be_bytes(data[4..8].try_into().unwrap());
    assert_eq!(version, 2);
    let objects = u32::from_be_bytes(data[8..].try_into().unwrap());
    return objects;
}

/// parses variable size encoding according to the specification gitformat-pack.txt (*1)
/// returns a tuple of the (bytes_read, encoded_size)
///
/// (*1) https://github.com/git/git/blob/795ea8776befc95ea2becd8020c7a284677b4161/Documentation/gitformat-pack.txt#L83
fn parse_size_encoding(data: &[u8], ix: usize, starting_shift: u8) -> (usize, usize) {
    let mut ix_ = ix.clone();
    let mut byte = data[ix_];
    let mut size: usize = usize::from(byte & (u8::from(2).pow(starting_shift.into()) - 1));
    let mut shift = starting_shift;
    while byte > 127 {
        ix_ += 1;
        byte = data[ix_];
        size |= usize::from(byte & 0b01111111) << shift;
        shift += 7
    }
    ix_ += 1;
    return (ix_ - ix, size);
}

fn apply_delta(data: &[u8], source_buf: &Bytes, target_size: usize) -> Vec<u8> {
    let mut buf = Cursor::new(data);
    let mut target_buf = Vec::new();
    while buf.remaining() > 0 {
        let b = buf.get_u8();
        let mut offset: usize = 0;
        let mut size: usize = 0;
        if (b >> 7) == 1 {
            // Copy mode
            if (b & 0b1) != 0 {
                offset |= buf.get_u8() as usize;
            }
            if (b & 0b10) != 0 {
                offset |= (buf.get_u8() as usize) << 8;
            }
            if (b & 0b100) != 0 {
                offset |= (buf.get_u8() as usize) << 16;
            }
            if (b & 0b1000) != 0 {
                offset |= (buf.get_u8() as usize) << 24;
            }
            if (b & 0b1_0000) != 0 {
                size |= buf.get_u8() as usize;
            }
            if (b & 0b10_0000) != 0 {
                size |= (buf.get_u8() as usize) << 8;
            }
            if (b & 0b100_0000) != 0 {
                size |= (buf.get_u8() as usize) << 16;
            }
            target_buf.append(&mut source_buf[offset..offset + size].to_vec());
        } else {
            // Add mode
            let mut data = vec![0u8; b as usize];
            buf.copy_to_slice(&mut data);
            target_buf.append(&mut data);
        }
    }
    assert_eq!(target_buf.len(), target_size);
    return target_buf;
}

fn read_hash(data: &[u8], ix: usize) -> String {
    return hex::encode(
        &data
            .get(ix..ix + 20)
            .expect("should have 20 bytes for sha1"),
    );
}

fn parse_entries(data: &[u8]) -> Vec<Entry> {
    let mut byhash: HashMap<String, Entry> = HashMap::new();
    let mut ix = 0;
    while ix < data.len() - 20 {
        let type_bytes = data[ix] & 0b01110000;
        let object_type = ObjectType::from_u8(type_bytes >> 4);
        let (bytes_read, size) = parse_size_encoding(&data, ix, 4);
        ix += bytes_read;
        match object_type {
            ObjectType::OfsDelta => {
                panic!("unsupported object type ofs-delta")
            }
            ObjectType::RefDelta => {
                let parent_sha = read_hash(&data, ix);
                ix += 20;

                let (bytes_read, content) = unpack_compressed_data(&data[ix..]).unwrap();
                let (source_len_bytes, source_len) = parse_size_encoding(&content, 0, 7);
                let (target_len_bytes, target_len) =
                    parse_size_encoding(&content, source_len_bytes, 7);
                ix += bytes_read;

                let header_bytes_read = target_len_bytes + source_len_bytes;
                let intructions = &content[header_bytes_read..];
                let prev_data = byhash.get(&parent_sha).unwrap();
                assert_eq!(prev_data.data.len(), source_len);

                let deltified = Bytes::from(apply_delta(intructions, &prev_data.data, target_len));
                let sha1 = calculate_object_hash(&prev_data.type_.to_string(), &deltified.to_vec());
                let entry = Entry {
                    type_: prev_data.type_,
                    size: target_len,
                    sha1: sha1.clone(),
                    data: deltified,
                };
                byhash.insert(sha1, entry);
            }
            _ => {
                let (bytes_read, content) = unpack_compressed_data(&data[ix..]).unwrap();
                ix += bytes_read;
                let sha1 = calculate_object_hash(&object_type.to_string(), &content.to_vec());
                let entry = Entry {
                    type_: object_type,
                    size: size,
                    sha1: sha1.clone(),
                    data: content,
                };
                byhash.insert(sha1, entry);
            }
        };
    }

    return byhash.values().cloned().collect();
}

pub fn parse_packfile(data: &[u8]) -> Packfile {
    let data = Bytes::from(data.to_vec());
    let objects = parse_header(&data[..12]);
    let entries = parse_entries(&data[12..]);
    let packhash = read_hash(&data, data.len() - 20);
    assert_eq!(objects as usize, entries.len());
    return Packfile {
        sha1: packhash,
        entries: entries,
    };
}
