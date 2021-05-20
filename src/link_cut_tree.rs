use crate::splay_forest::{SplayForest, NodeIdx};
use std::fmt::Debug;

pub struct LinkCutTree<V> {
    rep: SplayForest<V>,
}

impl<V: Debug> LinkCutTree<V> {
    pub fn new() -> Self {
        LinkCutTree { rep: SplayForest::new() }
    }

    pub fn make_tree(&mut self, val: V) -> NodeIdx {
        self.rep.add_node(val)
    }

    pub fn access(&mut self, node_idx: NodeIdx) {
        self.rep.splay(node_idx);
        self.rep.split_right_and_attach_new(node_idx, None);
        let mut v = node_idx;
        loop {
            match self.rep.get_path_parent(v) {
                None => { break; }
                Some(w) => {
                    self.rep.splay(w);
                    self.rep.split_right_and_attach_new(w, v.into());
                    v = w;
                }
            }
        }
        self.rep.splay(node_idx);
    }

    pub fn find_root(&mut self, node_idx: NodeIdx) -> NodeIdx {
        self.access(node_idx);
        let root = self.rep.get_leftmost(node_idx);
        self.access(root);
        return root;
    }

    pub fn cut(&mut self, node_idx: NodeIdx) {
        self.access(node_idx);
        self.rep.split_left(node_idx);
    }

    pub fn link(&mut self, parent_idx: NodeIdx, child_idx: NodeIdx) {
        self.access(child_idx);
        self.access(parent_idx);
        self.rep.join_left(child_idx, parent_idx);
    }

    pub fn get_val(&mut self, node_idx: NodeIdx) -> &V {
        self.rep.get_value(node_idx)
    }
}

#[cfg(test)]
mod tests {
    use crate::link_cut_tree::LinkCutTree;

    #[test]
    fn basic_tree() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        assert_eq!(lct.find_root(node1), node1);
    }

    #[test]
    fn multiple_roots() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        assert_eq!(lct.find_root(node1), node1);
        assert_eq!(lct.find_root(node2), node2);
        assert_eq!(lct.find_root(node1), node1);
    }

    #[test]
    fn link_basic() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        lct.link(node1, node2);
        assert_eq!(lct.find_root(node1), node1, "original root changed");
        assert_eq!(lct.find_root(node2), node1, "new root not updated");
        assert_eq!(lct.find_root(node1), node1, "original root changed after one iteration");
        assert_eq!(lct.find_root(node1), node1, "original root changed after repetition");
        assert_eq!(lct.find_root(node2), node1, "new root unupdated");
    }

    #[test]
    fn link_multiple() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("Grandparent");
        let node2 = lct.make_tree("Parent");
        let node3 = lct.make_tree("Child");
        lct.link(node1, node2);
        lct.link(node2, node3);
        assert_eq!(lct.find_root(node3), node1, "Didn't find grandparent");
        assert_eq!(lct.find_root(node2), node1, "Didn't find parent");
        assert_eq!(lct.find_root(node2), node1, "Didn't find itself");
    }

    #[test]
    fn many_links_to_one() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let node1 = lct.make_tree("Parent".into());
        let children = (1..6).map(
            |i| lct.make_tree(format!("Child {}", i))
        ).collect::<Vec<_>>();
        for child in &children {
            lct.link(node1, *child);
        }

        for child in &children {
            assert_eq!(lct.find_root(*child), node1, "Wrong parent");
        }
    }

    #[test]
    fn many_links_to_one_root() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let root = lct.make_tree("Root".into());
        let mut all_nodes = vec![root];
        let mut last_depth_nodes = vec![root];
        let mut current_depth_nodes = vec![];
        for _depth in 0..3 {
            for parent_idx in &last_depth_nodes {
                for child_num in 1..3 {
                    let child_name = format!("{}({})", lct.get_val(*parent_idx), child_num);
                    let child_idx = lct.make_tree(child_name);
                    lct.link(*parent_idx, child_idx);
                    current_depth_nodes.push(child_idx);
                    all_nodes.push(child_idx);
                }
            }
            last_depth_nodes.clear();
            last_depth_nodes.append(&mut current_depth_nodes);
        }

        for node in all_nodes {
            assert_eq!(lct.find_root(node), root, "Failed on node: {}", lct.get_val(node));
        }
    }

    #[test]
    fn test_cut_basic() {
        let mut lct: LinkCutTree<&str> = LinkCutTree::new();
        let node1 = lct.make_tree("1");
        let node2 = lct.make_tree("2");
        lct.link(node1, node2);
        assert_eq!(lct.find_root(node1), node1, "original root changed");
        assert_eq!(lct.find_root(node2), node1, "new root not updated");
        lct.cut(node2);
        assert_eq!(lct.find_root(node1), node1, "should still be its own root");
        assert_eq!(lct.find_root(node2), node2, "should be back to being its own root");
    }

    #[test]
    fn many_cut_links() {
        let mut lct: LinkCutTree<String> = LinkCutTree::new();
        let node1 = lct.make_tree("Parent".into());
        let children = (1..6).map(
            |i| lct.make_tree(format!("Child {}", i))
        ).collect::<Vec<_>>();
        for child in &children {
            lct.link(node1, *child);
        }

        for child in &children {
            lct.cut(*child)
        }

        for child in &children {
            assert_eq!(lct.find_root(*child), *child, "Wrong parent");
        }
        assert_eq!(lct.find_root(node1), node1);
    }

}