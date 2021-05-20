use crate::splay_forest::SplayForest;

mod splay_forest;

fn main() {
    let mut lct: SplayForest<&str> = SplayForest::new();
    let node1 = lct.add_node("alpha");
    let node2 = lct.add_node("bravo");
    let node3 = lct.add_node("charlie");
    lct.set_right(node1, Some(node2));
    lct.set_right(node2, Some(node3));


    let node4 = lct.add_node("A");
    let node5 = lct.add_node("B");
    let node6 = lct.add_node("C");
    lct.set_left(node4, Some(node5));
    lct.set_right(node5, Some(node6));

    println!("{:#?}", lct);

    lct.splay(node3);

    println!("{:#?}", lct);
}