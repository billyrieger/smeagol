use crate::{Cell, Node, NodeTemplate, Store};

impl Store {
    pub fn create_leaf(&self, cell: Cell) -> Node {
        Node::new_leaf(cell.is_alive())
    }

    pub fn create_level_one_from_cells(&self, cells: u8) -> Node {
        Node::new_level_one(cells)
    }

    pub fn create_level_two_from_cells(&mut self, cells: u16) -> Node {
        let node = Node::new_level_two(cells);
        let index = self.add_node(node, u128::from(cells.count_ones()));
        node.set_index(index)
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> Node {
        let level = template.ne.level();
        assert_eq!(level, template.nw.level());
        assert_eq!(level, template.se.level());
        assert_eq!(level, template.sw.level());

        match level {
            0 => {
                let (node, _) =
                    Node::create_level_one(template.ne, template.nw, template.se, template.sw);
                node
            }
            1 => {
                let (node, population) =
                    Node::create_level_two(template.ne, template.nw, template.se, template.sw);
                let index = self.add_node(node, population);
                node.set_index(index)
            }
            _ => {
                let ne_index = template.ne.index();
                let nw_index = template.nw.index();
                let se_index = template.se.index();
                let sw_index = template.sw.index();

                let population = self.populations[ne_index]
                    + self.populations[nw_index]
                    + self.populations[se_index]
                    + self.populations[sw_index];

                let node = Node::new_interior(level + 1, [ne_index, nw_index, se_index, sw_index]);
                let index = self.add_node(node, population);
                node.set_index(index)
            }
        }
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

    fn add_node(&mut self, node: Node, population: u128) -> usize {
        if let Some(index) = self.indices.get(&node).cloned() {
            index
        } else {
            let index = self.nodes.len();
            self.indices.insert(node, index);
            self.nodes.push(node.set_index(index));
            self.populations.push(population);
            index
        }
    }
}
