# Link-Cut Tree implementation in Rust

This repository contains a Rust implementation of the amortized logarithmic link-cut tree data structure as described in [this lecture](http://courses.csail.mit.edu/6.851/spring21/lectures/L19.htm).
It currently supports `link`, `cut`, and `find_root` operations; see [`src/link_cut_tree.rs`](src/link_cut_tree.rs) for the API.
Path aggregation (and better documentation) are still to-do items.
