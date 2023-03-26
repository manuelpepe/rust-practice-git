use crate::files::hashobject;
use crate::objects::{load_object, store_object, GitObject, ObjectHeader};
use sha1::Digest;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::{collections::HashMap, slice::Iter};

#[derive(Debug)]
pub struct TreeNode {
    pub permissions: String, // TODO: Change to hex
    pub filename: String,
    pub hash: String,
}

fn parse_tree(tree: &GitObject) -> Vec<TreeNode> {
    if let GitObject::Tree { len: _, data } = tree {
        let mut vec: Vec<TreeNode> = Vec::new();
        for line in data.strip_suffix('\n').unwrap_or(data).split('\n') {
            let mut iter = line.split('\t');
            vec.push(TreeNode {
                permissions: iter.next().unwrap().to_string(),
                filename: iter.next().unwrap().to_string(),
                hash: iter.next().unwrap().to_string(),
            })
        }
        return vec;
    }
    panic!("object not a tree")
}

#[derive(Debug)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
}

impl Tree {
    fn new(tree: &GitObject) -> Tree {
        return Tree {
            nodes: parse_tree(&tree),
        };
    }

    pub fn iter(&self) -> Iter<TreeNode> {
        return self.nodes.iter();
    }

    pub fn to_buf(&self, buf: &mut Vec<u8>) {
        for node in &self.nodes {
            let mut permission = node.permissions.as_bytes().to_vec();
            let mut filename = node.filename.as_bytes().to_vec();
            let mut hash = node.hash.as_bytes().to_vec();
            buf.append(&mut permission);
            buf.push(0x20);
            buf.append(&mut filename);
            buf.push(0);
            buf.append(&mut hex::decode(&mut hash).unwrap());
        }
    }
}

pub fn lstree(treeid: &String) -> Tree {
    let obj = load_object(treeid);
    let tree = Tree::new(&obj);
    return tree;
}

pub fn writetree() {
    let hash = hash_current_dir().unwrap();
    println!("{}", hash);
}

fn visit_dirs(vec: &mut Vec<PathBuf>, dir: &PathBuf) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && !path.starts_with("./.git") {
                vec.push(path.clone());
                visit_dirs(vec, &path)?;
            }
        }
    }
    Ok(())
}

// TODO: Refactor, start from scenario without subdirs and recurse upwards.
fn hash_current_dir() -> io::Result<String> {
    let mut to_scan: Vec<PathBuf> = Vec::new();
    visit_dirs(&mut to_scan, &PathBuf::from("."))?;
    to_scan.reverse();
    to_scan.push(PathBuf::from("./"));

    let mut scanned: HashMap<String, String> = HashMap::new();
    for subd in to_scan {
        let mut hash = sha1::Sha1::new();
        let mut tree = Tree { nodes: Vec::new() };
        let subd_string = subd.as_os_str().to_str().unwrap();
        hash.update(subd_string);

        for node in fs::read_dir(&subd)? {
            let node = node?;
            let path = node.path();
            if path.starts_with("./.git") {
                continue;
            }

            let path_string = path.as_os_str().to_str().unwrap();
            if path.is_dir() {
                hash.update(scanned.get(path_string).unwrap());
                tree.nodes.push(TreeNode {
                    permissions: "40000".to_string(),
                    filename: node.file_name().to_str().unwrap().to_string(),
                    hash: scanned.get(path_string).unwrap().to_string(),
                });
            } else {
                hash.update(hashobject(&path_string.to_string(), false));
                tree.nodes.push(TreeNode {
                    permissions: "100644".to_string(),
                    filename: node.file_name().to_str().unwrap().to_string(),
                    hash: hashobject(&path_string.to_string(), false),
                });
            }
            // hash.update(node.metadata()?.permissions());
        }
        let tree_hash = format!("{:x}", hash.finalize());
        let mut buf = Vec::new();
        tree.to_buf(&mut buf);
        store_object(
            &buf,
            &tree_hash,
            ObjectHeader {
                type_: "tree".to_string(),
                len: buf.len(),
            },
        );
        scanned.insert(subd_string.to_string(), tree_hash);
    }
    return Ok(scanned.get("./").unwrap().to_string());
}
