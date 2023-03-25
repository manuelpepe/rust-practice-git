use crate::objects::{load_object, GitObject};
use std::slice::Iter;

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
}

pub fn lstree(treeid: &String) -> Tree {
    let obj = load_object(treeid);
    let tree = Tree::new(&obj);
    return tree;
}
