use crate::{Cell, Node, NodeTemplate, Store};

impl Store {
    pub fn create_leaf(&mut self, cell: Cell) -> Node {
        let node = Node::new_leaf(cell);
        self.add_node(node, if cell.is_alive() { 1 } else { 0 });
        node
    }

    pub fn create_level_one_from_cells(&mut self, cells: u8) -> Node {
        let node = Node::new_level_one(cells);
        self.add_node(node, u128::from(cells.count_ones()));
        node
    }

    pub fn create_level_two_from_cells(&mut self, cells: u16) -> Node {
        let node = Node::new_level_two(cells);
        self.add_node(node, u128::from(cells.count_ones()));
        node
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> Node {
        let level = template.ne.level();
        assert_eq!(level, template.nw.level());
        assert_eq!(level, template.se.level());
        assert_eq!(level, template.sw.level());

        match level {
            0 => {
                let mut cells = 0u8;
                if template.nw.population(&self) == 1 {
                    cells |= 0b_0010_0000;
                }
                if template.ne.population(&self) == 1 {
                    cells |= 0b_0001_0000;
                }
                if template.sw.population(&self) == 1 {
                    cells |= 0b_0000_0010;
                }
                if template.se.population(&self) == 1 {
                    cells |= 0b_0000_0001;
                }

                let node = Node::new_level_one(cells);
                self.add_node(node, u128::from(cells.count_ones()));
                node
            }
            1 => {
                let (node, population) =
                    Node::create_level_two(template.ne, template.nw, template.se, template.sw);
                self.add_node(node, population);
                node
            }
            _ => {
                let ne_index = self.indices[&template.ne];
                let nw_index = self.indices[&template.nw];
                let se_index = self.indices[&template.se];
                let sw_index = self.indices[&template.sw];

                let population = self.populations[ne_index]
                    + self.populations[nw_index]
                    + self.populations[se_index]
                    + self.populations[sw_index];

                let node = Node::new_interior(level + 1, [ne_index, nw_index, se_index, sw_index]);
                self.add_node(node, population);
                node
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

    fn add_node(&mut self, node: Node, population: u128) {
        if !self.indices.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.push(node);
            self.populations.push(population);
            self.indices.insert(node, index);
        }
    }
}
