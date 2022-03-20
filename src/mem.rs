use crate::life::{Branch, Node};
use crate::util::{Grid2, ToGrid};
use indexmap::IndexMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: Idx,
    generation: Gen,
}

impl NodeId {
    pub fn new(idx: Idx, gen: Gen) -> Self {
        Self {
            index: idx,
            generation: gen,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Idx(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gen(u32);

pub struct Entry<'a, T> {
    pub node: &'a Node,
    pub data: &'a T,
}

pub struct NodeArena<T> {
    generation: Gen,
    nodes: IndexMap<Node, T, fnv::FnvBuildHasher>,
}

impl<T> NodeArena<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            generation: Gen(0),
            nodes: IndexMap::default(),
        }
    }

    pub fn entry(&self, id: NodeId) -> Option<Entry<T>> {
        self.nodes
            .get_index(id.index.0 as usize)
            .map(|(node, data)| Entry { node, data })
    }

    pub fn get(&self, id: NodeId) -> Option<(&Node, &T)> {
        self.nodes.get_index(id.index.0 as usize)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.get(id).map(|(k, _)| k)
    }

    pub fn get_data(&self, id: NodeId) -> Option<&T> {
        self.get(id).map(|(_, v)| v)
    }

    pub fn insert(&mut self, node: Node) -> NodeId {
        let entry = self.nodes.entry(node);
        let index = Idx(entry.index() as u32);
        // Actually insert the node into the indexmap.
        entry.or_default();
        NodeId {
            index,
            generation: self.generation,
        }
    }

    pub fn make_branch(&mut self, child_ids: Grid2<NodeId>) -> Option<NodeId> {
        let nodes: [&Node; 4] = child_ids.to_array().try_map(|id| self.get_node(id))?;
        let child_side_log2 = nodes[0].side_log2();
        assert!(nodes.iter().all(|node| node.side_log2() == child_side_log2));
        let branch = Branch {
            is_empty: nodes.iter().all(|node| node.is_empty()),
            side_log2: child_side_log2 + 1,
            gen: self.generation,
            children: child_ids.to_array().map(|id| id.index).to_grid(),
        };
        Some(self.insert(Node::Branch(branch)))
    }
}
