use crate::{Cell, Node, NodeTemplate, Store};
use rand::distributions::Distribution;

/// Methods to create a `Node`.
impl Store {
    /// Creates a leaf node corresponding to the given cell.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    ///
    /// let alive = store.create_leaf(smeagol::Cell::Alive);
    /// assert_eq!(alive.population(&mut store), 1);
    /// ```
    pub fn create_leaf(&mut self, cell: Cell) -> Node {
        let node = Node::new_leaf(cell);
        self.add_node(node);
        node
    }

    /// Creates a new interior node with the given children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the levels of the nodes are not all equal.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::Store::new();
    ///
    /// let node = store
    ///     .create_empty(5);
    /// // +---+---+---+---+
    /// // |   |   |   |   |
    /// // +---+---+---+---+
    /// // |   | * | * |   |
    /// // +---+---+---+---+
    /// // |   | * | * |   |
    /// // +---+---+---+---+
    /// // |   |   |   |   |
    /// // +---+---+---+---+
    /// let center_subnode = store.create_interior(smeagol::NodeTemplate {
    ///     ne: node.ne(&store).sw(&store),
    ///     nw: node.nw(&store).se(&store),
    ///     se: node.se(&store).nw(&store),
    ///     sw: node.sw(&store).ne(&store),
    /// });
    /// assert_eq!(center_subnode, node.center_subnode(&mut store));
    /// ```
    pub fn create_interior(&mut self, template: NodeTemplate) -> Node {
        assert_eq!(template.ne.level(), template.nw.level());
        assert_eq!(template.ne.level(), template.se.level());
        assert_eq!(template.ne.level(), template.sw.level());

        let ne_index = self.indices[&template.ne];
        let nw_index = self.indices[&template.nw];
        let se_index = self.indices[&template.se];
        let sw_index = self.indices[&template.sw];

        let level = template.ne.level() + 1;
        let contains_alive_cells = self.node(ne_index).contains_alive_cells()
            || self.node(nw_index).contains_alive_cells()
            || self.node(se_index).contains_alive_cells()
            || self.node(sw_index).contains_alive_cells();

        let node = Node::new_interior(level, [ne_index, nw_index, se_index, sw_index], contains_alive_cells);
        self.add_node(node);
        node
    }

    /// Creates a new node with the given level filled with only dead cells.
    ///
    /// # Panics
    ///
    /// Panics if `level > MAX_LEVEL`.
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

    pub fn create_random(&mut self, level: u8, fill: f64) -> Node {
        assert!(0. <= fill);
        assert!(fill <= 1.);

        let mut rng = rand::thread_rng();
        let distribution = rand::distributions::Uniform::new(0., 1.);

        if level == 0 {
            if distribution.sample(&mut rng) < fill {
                self.create_leaf(Cell::Alive)
            } else {
                self.create_leaf(Cell::Dead)
            }
        } else {
            let ne = self.create_random(level - 1, fill);
            let nw = self.create_random(level - 1, fill);
            let se = self.create_random(level - 1, fill);
            let sw = self.create_random(level - 1, fill);
            self.create_interior(NodeTemplate { ne, nw, se, sw })
        }
    }

    fn add_node(&mut self, node: Node) {
        if !self.indices.contains_key(&node) {
            let index = self.nodes.len();
            self.nodes.push(node);
            self.indices.insert(node, index);
        }
    }
}
