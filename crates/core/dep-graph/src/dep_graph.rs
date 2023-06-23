use crate::errors::DepGraphError;
use moon_action::ActionNode;
use petgraph::algo::toposort;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use rustc_hash::{FxHashMap, FxHashSet};

pub type DepGraphType = DiGraph<ActionNode, ()>;
pub type IndicesType = FxHashMap<ActionNode, NodeIndex>;
pub type BatchedTopoSort = Vec<Vec<NodeIndex>>;

/// A directed acyclic graph (DAG) for the work that needs to be processed, based on a
/// project or task's dependency chain. This is also known as a "task graph" (not to
/// be confused with our tasks) or a "dependency graph".
pub struct DepGraph {
    graph: DepGraphType,

    indices: IndicesType,
}

impl DepGraph {
    pub fn new(graph: DepGraphType, indices: IndicesType) -> Self {
        DepGraph { graph, indices }
    }

    pub fn get_index_from_node(&self, node: &ActionNode) -> Option<&NodeIndex> {
        self.indices.get(node)
    }

    pub fn get_node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn get_node_from_index(&self, index: &NodeIndex) -> Option<&ActionNode> {
        self.graph.node_weight(*index)
    }

    pub fn sort_topological(&self) -> Result<Vec<NodeIndex>, DepGraphError> {
        let list = match toposort(&self.graph, None) {
            Ok(nodes) => nodes,
            Err(error) => {
                return Err(DepGraphError::CycleDetected(
                    self.get_node_from_index(&error.node_id()).unwrap().label(),
                ));
            }
        };

        Ok(list.into_iter().rev().collect())
    }

    pub fn sort_batched_topological(&self) -> Result<BatchedTopoSort, DepGraphError> {
        let mut batches: BatchedTopoSort = vec![];

        // Count how many times an index is referenced across nodes and edges
        let mut node_counts = FxHashMap::<NodeIndex, u32>::default();

        for ix in self.graph.node_indices() {
            node_counts.entry(ix).and_modify(|e| *e += 1).or_insert(0);

            for dep_ix in self.graph.neighbors(ix) {
                node_counts
                    .entry(dep_ix)
                    .and_modify(|e| *e += 1)
                    .or_insert(0);
            }
        }

        // Gather root nodes (count of 0)
        let mut root_nodes = FxHashSet::<NodeIndex>::default();

        for (ix, count) in &node_counts {
            if *count == 0 {
                root_nodes.insert(*ix);
            }
        }

        // If no root nodes are found, but nodes exist, then we have a cycle
        let has_sync_workspace = root_nodes.contains(&NodeIndex::new(0));

        if (!has_sync_workspace && root_nodes.is_empty()
            || has_sync_workspace && root_nodes.len() == 1)
            && !node_counts.is_empty()
        {
            self.detect_cycle()?;
        }

        while !root_nodes.is_empty() {
            let mut next_root_nodes = FxHashSet::<NodeIndex>::default();

            // Decrement dependencies of the current batch nodes
            for ix in &root_nodes {
                for dep_ix in self.graph.neighbors(*ix) {
                    let count = node_counts
                        .entry(dep_ix)
                        .and_modify(|e| *e -= 1)
                        .or_insert(0);

                    // And create a new batch if the count is 0
                    if *count == 0 {
                        next_root_nodes.insert(dep_ix);
                    }
                }
            }

            // Push the previous batch onto the list
            batches.push(root_nodes.into_iter().collect());

            // And reset the current nodes
            root_nodes = next_root_nodes;
        }

        // Move persistent targets to the end
        let mut sorted_batches: BatchedTopoSort = vec![];
        let mut persistent: Vec<NodeIndex> = vec![];

        for mut batch in batches.into_iter().rev() {
            batch.retain(|ix| match self.graph.node_weight(*ix).unwrap() {
                ActionNode::RunPersistentTarget(_, _) => {
                    persistent.push(*ix);
                    false
                }
                _ => true,
            });

            if !batch.is_empty() {
                sorted_batches.push(batch);
            }
        }

        if !persistent.is_empty() {
            sorted_batches.push(persistent);
        }

        Ok(sorted_batches)
    }

    /// Get a labelled representation of the dep graph (which can be serialized easily).
    pub fn labeled_graph(&self) -> DiGraph<String, ()> {
        let graph = self.graph.clone();
        graph.map(|_, n| n.label(), |_, e| *e)
    }

    pub fn to_dot(&self) -> String {
        let graph = self.graph.map(|_, n| n.label(), |_, e| e);

        let dot = Dot::with_attr_getters(
            &graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
            &|_, e| {
                if e.source().index() == 0 {
                    String::from("arrowhead=none")
                } else {
                    String::from("arrowhead=box, arrowtail=box")
                }
            },
            &|_, n| {
                let id = n.1;

                format!("label=\"{id}\" style=filled, shape=oval, fillcolor=gray, fontcolor=black")
            },
        );

        format!("{dot:?}")
    }

    #[track_caller]
    fn detect_cycle(&self) -> Result<(), DepGraphError> {
        use petgraph::algo::kosaraju_scc;

        let scc = kosaraju_scc(&self.graph);

        // Remove the sync workspace node
        let scc = scc
            .into_iter()
            .filter(|list| !(list.len() == 1 && list[0].index() == 0))
            .collect::<Vec<Vec<NodeIndex>>>();

        // The cycle is always the last sequence in the list
        let Some(cycle) = scc.last() else {
            return Err(DepGraphError::CycleDetected("(unknown)".into()));
        };

        let path = cycle
            .iter()
            .filter_map(|i| self.get_node_from_index(i).map(|n| n.label()))
            .collect::<Vec<String>>()
            .join(" → ");

        Err(DepGraphError::CycleDetected(path))
    }
}
