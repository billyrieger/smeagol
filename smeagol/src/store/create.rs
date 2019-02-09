use crate::{Cell, Node, NodeTemplate, Store};
use rand::Rng;

/// Methods to create nodes.
impl Store {
    /// Creates a leaf node corresponding to the given cell.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    /// let alive = store.create_leaf(smeagol::Cell::Alive);
    /// let dead = store.create_leaf(smeagol::Cell::Dead);
    /// ```
    pub fn create_leaf(&self, cell: Cell) -> Node {
        Node::new_leaf(cell.is_alive())
    }

    /// Creates a level 1 node corresponding to the given cells.
    ///
    /// The `u8` `00ab_00cd` corresponds to the level 1 node
    ///
    /// ```txt
    /// +---+---+
    /// | a | b |
    /// +---+---+
    /// | c | d |
    /// +---+---+
    /// ```
    ///
    /// where 1 represents an alive cell and 0 represents a dead cell.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    /// let block = store.create_level_one_from_cells(0b0011_0011);
    /// assert_eq!(block.population(&store), 4);
    /// ```
    pub fn create_level_one_from_cells(&self, cells: u8) -> Node {
        Node::new_level_one(cells)
    }

    /// Creates a level 2 node corresponding to the given cells.
    ///
    /// The `u16` `abcd_efgh_ijkl_mnop` correspond to the level 2 node
    ///
    /// ```txt
    /// +---+---+---+---+
    /// | a | b | c | d |
    /// +---+---+---+---+
    /// | e | f | g | h |
    /// +---+---+---+---+
    /// | i | j | k | l |
    /// +---+---+---+---+
    /// | m | n | o | p |
    /// +---+---+---+---+
    /// ```
    ///
    /// where 1 represents an alive cell and 0 represents a dead cell.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    /// let stripes = store.create_level_two_from_cells(0b1010_1010_1010_1010);
    /// assert_eq!(stripes.population(&store), 8);
    /// ```
    pub fn create_level_two_from_cells(&mut self, cells: u16) -> Node {
        let node = Node::new_level_two(cells);
        self.add_node(node, u128::from(cells.count_ones()))
    }

    /// Creates a node from the four children nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    ///
    /// let ne = store.create_random_filled(3, 0.5);
    /// let nw = store.create_random_filled(3, 0.5);
    /// let se = store.create_random_filled(3, 0.5);
    /// let sw = store.create_random_filled(3, 0.5);
    ///
    /// let random = store.create_interior(smeagol::NodeTemplate { ne, nw, se, sw });
    /// assert_eq!(random.level(), 4);
    /// ```
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
                self.add_node(node, population)
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
                self.add_node(node, population)
            }
        }
    }

    /// Creates an empty node of the given level.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    ///
    /// // 16 by 16 grid of dead cells
    /// let empty = store.create_empty(4);
    /// assert_eq!(empty.population(&store), 0);
    /// ```
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

    pub fn create_random_filled(&mut self, level: u8, fill: f64) -> Node {
        assert!(0. <= fill);
        assert!(fill <= 1.);

        let mut rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::new(0., 1.);

        if level == 0 {
            if rng.sample(dist) < fill {
                self.create_leaf(Cell::Alive)
            } else {
                self.create_leaf(Cell::Dead)
            }
        } else {
            let ne = self.create_random_filled(level - 1, fill);
            let nw = self.create_random_filled(level - 1, fill);
            let se = self.create_random_filled(level - 1, fill);
            let sw = self.create_random_filled(level - 1, fill);
            self.create_interior(NodeTemplate { ne, nw, se, sw })
        }
    }

    /// Adds a node to the store.
    ///
    /// This should be called whenever a level 2 or above node is created.
    fn add_node(&mut self, node: Node, population: u128) -> Node {
        if let Some(index) = self.indices.get(&node).cloned() {
            node.set_index(index)
        } else {
            let index = self.nodes.len();
            self.indices.insert(node, index);
            self.nodes.push(node.set_index(index));
            self.populations.push(population);
            node.set_index(index)
        }
    }
}
