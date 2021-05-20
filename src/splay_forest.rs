use std::num::NonZeroUsize;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::collections::HashSet;

pub trait SliceExt {
    type Item;

    fn get_two_mut(&mut self, index0: usize, index1: usize) -> (&mut Self::Item, &mut Self::Item);
}

impl<T> SliceExt for [T] {
    type Item = T;

    fn get_two_mut(&mut self, a: usize, b: usize) -> (&mut Self::Item, &mut Self::Item) {
        assert_ne!(a, b);
        assert!(a <= self.len());
        assert!(b <= self.len());
        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(self.get_unchecked_mut(a) as *mut _);
            let br = &mut *(self.get_unchecked_mut(b) as *mut _);
            (ar, br)
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct NodeIdx(NonZeroUsize);

impl NodeIdx {
    #[inline]
    pub fn new(idx: usize) -> Self {
        NodeIdx(NonZeroUsize::try_from(idx + 1).unwrap())
    }

    #[inline]
    fn get(self) -> usize {
        self.0.get() - 1
    }
}

impl From<usize> for NodeIdx {
    fn from(idx: usize) -> Self {
        NodeIdx::new(idx)
    }
}

#[derive(Debug)]
pub struct Node<V> {
    value: V,
    path_parent: Option<NodeIdx>,
    parent: Option<NodeIdx>,
    left: Option<NodeIdx>,
    right: Option<NodeIdx>,
    my_idx: NodeIdx
}

struct NodeDebug<'a, V> {
    forest: &'a SplayForest<V>,
    idx: Option<NodeIdx>
}

impl<'a, V: Debug> Debug for NodeDebug<'a, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.idx {
            None => {
                let none : Option<()> = None;
                none.fmt(f)
            }
            Some(idx) => {
                let mut debug_struct = f.debug_struct("Node");
                debug_struct.field("Idx", &idx.get());
                if let Some(path_parent_idx) = self.forest.get_node(idx).path_parent {
                    debug_struct.field("PathParent", &path_parent_idx.get());
                }
                debug_struct.field("Val", &self.forest.get_node(idx).value);
                let left_child = self.forest.get_node(idx).left;
                let left_debug = NodeDebug {forest: self.forest, idx: left_child};
                debug_struct.field("Left", &left_debug);
                let right_child = self.forest.get_node(idx).right;
                let right_debug = NodeDebug {forest: self.forest, idx: right_child};
                debug_struct.field("Right", &right_debug);
                debug_struct.finish()
            }
        }
    }
}

impl<V> Node<V> {
    pub fn new(value: V, cur_idx: NodeIdx) -> Self {
        Node {  value, path_parent: None, parent: None, left: None, right: None, my_idx: cur_idx }
    }
}

pub struct SplayForest<V> {
    pub forest: Vec<Node<V>>
}

impl<V> SplayForest<V> {
    pub fn new() -> Self {
        SplayForest { forest: Vec::new(), }
    }

    pub fn add_node(&mut self, node: V) -> NodeIdx {
        let cur_idx = self.forest.len().into();
        let node = Node::new(node, cur_idx);
        self.forest.push(node);
        cur_idx
    }

    fn get_node(&self, node_idx: NodeIdx) -> &Node<V> {
        &self.forest[node_idx.get()]
    }

    fn get_node_mut(&mut self, node_idx: NodeIdx) -> &mut Node<V> {
        &mut self.forest[node_idx.get()]
    }

    pub fn get_value(&self, node_idx: NodeIdx) -> &V {
        &self.get_node(node_idx).value
    }

    pub fn get_parent(&self, node_idx: NodeIdx) -> Option<NodeIdx> {
        self.get_node(node_idx).parent
    }

    pub fn get_path_parent(&self, node_idx: NodeIdx) -> Option<NodeIdx> {
        self.get_node(node_idx).path_parent
    }
    
    pub fn get_left(&self, node_idx: NodeIdx) -> Option<NodeIdx> {
        self.get_node(node_idx).left
    }
    
    pub fn get_right(&self, node_idx: NodeIdx) -> Option<NodeIdx> {
        self.get_node(node_idx).right
    }

    fn get_while_some<F>(&self, node_idx: NodeIdx, func: F) -> NodeIdx
        where F: Fn(NodeIdx) -> Option<NodeIdx> {
        let mut current_node_idx = node_idx;
        loop {
            match func(current_node_idx) {
                None => {
                    return current_node_idx;
                }
                Some(next_node_idx) => {
                    current_node_idx = next_node_idx;
                }
            }
        }
    }
    
    pub fn get_root(&self, node_idx: NodeIdx) -> NodeIdx {
        self.get_while_some(node_idx, |node_idx| self.get_parent(node_idx))
    }

    pub fn get_leftmost(&self, node_idx: NodeIdx) -> NodeIdx {
        self.get_while_some(node_idx, |node_idx| self.get_left(node_idx))
    }

    pub fn get_rightmost(&self, node_idx: NodeIdx) -> NodeIdx {
        self.get_while_some(node_idx, |node_idx| self.get_right(node_idx))
    }

    fn is_left_child(&self, node_idx: NodeIdx) -> bool {
        match self.get_parent(node_idx).map(|idx| self.get_node(idx)) {
            None => false,
            Some(parent) => match parent.left {
                None => false,
                Some(left_idx) => { left_idx.get() == node_idx.get() }
            }
        }
    }

    fn update_parent(&mut self, old_root_idx: NodeIdx, new_root_idx: NodeIdx) {
        let b_left = self.is_left_child(old_root_idx);
        match self.get_parent(old_root_idx) {
            None => {
                self.forest[new_root_idx.get()].parent = None;
                // Only need to update path parent if original was the root, as only the root has the path_parent
                self.forest[new_root_idx.get()].path_parent = self.forest[old_root_idx.get()].path_parent;
                self.forest[old_root_idx.get()].path_parent = None;
            }
            Some(parent_idx) => {
                if b_left {
                    self.set_left(parent_idx, Some(new_root_idx))
                } else {
                    self.set_right(parent_idx, Some(new_root_idx));
                }
            }
        }
    }

    fn rotate_right(&mut self, root_idx: NodeIdx) {
        let left_idx = self.get_node(root_idx).left;
        match left_idx {
            None => {}
            Some(new_root_idx) => {
                self.set_left(root_idx, self.get_node(new_root_idx).right);
                self.update_parent(root_idx, new_root_idx);
                self.set_right(new_root_idx, Some(root_idx));
            }
        }
    }

    fn rotate_left(&mut self, root_idx: NodeIdx) {
        let right_idx = self.get_node(root_idx).right;
        match right_idx {
            None => {}
            Some(new_root_idx) => {
                self.set_right(root_idx, self.get_node(new_root_idx).left);
                self.update_parent(root_idx, new_root_idx);
                self.set_left(new_root_idx, Some(root_idx));
            }
        }
    }

    pub fn rotate_up(&mut self, node_idx: NodeIdx) {
        let b_left = self.is_left_child(node_idx);
        let parent = self.get_parent(node_idx);

        match parent {
            None => {}
            Some(parent_idx) => {
                if b_left {
                    self.rotate_right(parent_idx);
                } else {
                    self.rotate_left(parent_idx);
                }
            }
        }
    }

    fn rotate_down(&mut self, node_idx: NodeIdx) {
        if self.get_node(node_idx).left.is_some() {
            self.rotate_right(node_idx);
        } else if self.get_node(node_idx).right.is_some() {
            self.rotate_left(node_idx);
        }
    }

    pub fn set_right(&mut self, node_idx: NodeIdx, right_idx: Option<NodeIdx>) {
        self.forest[node_idx.get()].right = right_idx;
        match right_idx {
            None => {}
            Some(idx) => {
                self.forest[idx.get()].parent = Some(node_idx);
            }
        }

    }

    pub fn set_left(&mut self, node_idx: NodeIdx, left_idx: Option<NodeIdx>) {
        self.forest[node_idx.get()].left = left_idx;
        match left_idx {
            None => {}
            Some(idx) => {
                self.forest[idx.get()].parent = Some(node_idx);
            }
        }
    }


    pub fn splay(&mut self, node_idx: NodeIdx) {
        enum SplayType {NoSplay, Zig, ZigZig, ZigZag}

        loop {
            let splay_type = match self.get_parent(node_idx) {
                None => SplayType::NoSplay,
                Some(parent_idx) => match self.get_parent(parent_idx) {
                    None => SplayType::Zig,
                    Some(_) => if self.is_left_child(node_idx) == self.is_left_child(parent_idx) {
                        SplayType::ZigZig
                    } else {
                        SplayType::ZigZag
                    }
                }
            };

            match splay_type {
                SplayType::NoSplay => return,
                SplayType::Zig => self.rotate_up(node_idx),
                SplayType::ZigZag => {
                    self.rotate_up(node_idx);
                    self.rotate_up(node_idx);
                }
                SplayType::ZigZig => {
                    self.rotate_up(self.get_parent(node_idx).unwrap());
                    self.rotate_up(node_idx)
                }
            }
        }
    }

    pub fn split_right_and_attach_new(&mut self, node_idx: NodeIdx, new_right_idx: Option<NodeIdx>) {
        if let Some(right_idx) = self.get_right(node_idx) {
            let right_node = self.get_node_mut(right_idx);
            right_node.path_parent = node_idx.into();
            right_node.parent = None;
        }

        self.get_node_mut(node_idx).right = new_right_idx;
        if let Some(new_right_index) = new_right_idx {
            let new_right = self.get_node_mut(new_right_index);
            new_right.parent = node_idx.into();
            new_right.path_parent = None;
        }
    }

    pub fn split_left(&mut self, node_idx: NodeIdx) {
        if let Some(left_idx) = self.get_left(node_idx) {
            self.get_node_mut(left_idx).parent = None;
            self.get_node_mut(node_idx).left = None;
        }
    }

    pub fn join_left(&mut self, node_idx: NodeIdx, left_idx: NodeIdx) {
        assert_eq!(self.get_node_mut(node_idx).left, None);
        self.get_node_mut(node_idx).left = left_idx.into();
        self.get_node_mut(left_idx).parent = node_idx.into();
    }
}

impl<V: Debug> Debug for SplayForest<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut roots = HashSet::new();
        for i in 0..self.forest.len() {
            if self.get_node(NodeIdx::new(i)).parent.is_none() {
                roots.insert(NodeIdx::new(i));
            }
        }
        let mut dbg = f.debug_list();
        for root in roots.into_iter() {
            let debug_node = NodeDebug{forest: &self, idx: Some(root)};
            dbg.entry(&debug_node);
        }
        dbg.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::splay_forest::SplayForest;

    #[test]
    fn it_works() {
        let mut lct: SplayForest<&str> = SplayForest::new();

        let node1 = lct.add_node("alpha");
        lct.rotate_down(node1);
        let node2 = lct.add_node("bravo");
        let node3 = lct.add_node("charlie");
        lct.set_left(node1, node2.into());
        lct.set_right(node2, Some(node3));

        println!("{:#?}", lct);

        lct.splay(node3);

        println!("{:#?}", lct);
    }
}