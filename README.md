# Git in Rust

Made for practice following the [CodeCrafter's Rust track](https://app.codecrafters.io/courses/git).


## Implemented Features


* `init`: Initialize git repository (creates basic `.git`)
* `hash-object [-w] <filepath>`: Store a blob object in `.git/objects`
* `cat-file <-p> <sha1>`: Prints content of blob object
* `ls-tree [--name-only] <sha1>`: Prints content of tree object
* `write-tree`: Stores the whole current directory as a tree object in `.git/objects`. All subdirectories and files are also stored as trees and blobs respectively.
* `commit-tree <tree_sha> -p <commit_sha> -m <message>`: Store a commit object in `.git/objects`

## TODO: 

* Implement `git pull`
* Better error handling
* Better argument parsing
* Add file type to `ls-tree` output without `--name-only`