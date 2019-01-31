use crate::{Cell, Node, NodeTemplate, Store};

impl Store {
    pub fn create_leaf(&mut self, cell: Cell) -> Node {
        let node = Node::new_leaf(cell);
        self.add_node(node, if cell.is_alive() { 1 } else { 0 });
        node
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> Node {
        assert_eq!(template.ne.level(), template.nw.level());
        assert_eq!(template.ne.level(), template.se.level());
        assert_eq!(template.ne.level(), template.sw.level());

        let ne_index = self.indices[&template.ne];
        let nw_index = self.indices[&template.nw];
        let se_index = self.indices[&template.se];
        let sw_index = self.indices[&template.sw];

        let level = template.ne.level() + 1;
        let population = self.populations[ne_index]
            + self.populations[nw_index]
            + self.populations[se_index]
            + self.populations[sw_index];

        let node = Node::new_interior(level, [ne_index, nw_index, se_index, sw_index]);
        self.add_node(node, population);
        node
    }

    pub fn create_empty(&mut self, level: u8) -> Node {
        if level == 0 {
            self.create_leaf(Cell::Dead)
        } else {
            let empty = self.create_empty(level - 1);
            self.create_interior(NodeTemplate {
                ne: empty,
                nw: empty,
                se: empty,
                sw: empty,
            })
        }
    }

    fn add_node(&mut self, node: Node, population: u128) {
        if !self.indices.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.push(node);
            self.populations.push(population);
            self.indices.insert(node, index);
        }
    }
}
