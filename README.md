# Git in Rust

Made for practicing rust and bit operations following the [CodeCrafter's Rust track](https://app.codecrafters.io/courses/git?track=rust).


## Implemented Features


* `init`: Initialize git repository (creates basic `.git`)
* `hash-object [-w] <filepath>`: Store a blob object in `.git/objects`
* `cat-file <-p> <sha1>`: Prints content of blob object
* `ls-tree [--name-only] <sha1>`: Prints content of tree object
* `write-tree`: Stores the whole current directory as a tree object in `.git/objects`. All subdirectories and files are also stored as trees and blobs respectively.
* `commit-tree <tree_sha> -p <commit_sha> -m <message>`: Store a commit object in `.git/objects`
* `clone <url> <dir>`: Clone a repository

## TODO: 

* ~~Implement `git clone`~~
* ~~Better error handling~~
* ~~Better argument parsing~~
* ~~Add tests~~
* Add object type to `ls-tree` output without `--name-only`