use nodegraph::NodeGraph;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Transform {
    position: (f32, f32), // 2D x, y coordinates
    rotation: f32,        // Rotation in degrees
}

impl Transform {
    fn new(x: f32, y: f32, rotation: f32) -> Self {
        Self {
            position: (x, y),
            rotation,
        }
    }

    fn aggregate(&self, parent: &Transform) -> Transform {
        Self {
            position: (
                self.position.0 + parent.position.0,
                self.position.1 + parent.position.1,
            ),
            rotation: self.rotation + parent.rotation,
        }
    }
}

struct SceneGraph<ID: Clone + Eq + Hash + Ord + Debug> {
    graph: NodeGraph<ID, (), Transform>,
    root_id: ID,
}

impl<ID: Clone + Eq + Hash + Ord + Debug> SceneGraph<ID> {
    fn new(root_id: ID) -> Self {
        let mut graph = NodeGraph::new();
        graph.add_node(root_id.clone(), Transform::new(0.0, 0.0, 0.0));

        SceneGraph { graph, root_id }
    }

    fn add_child(
        &mut self,
        child_id: ID,
        parent_id: ID,
        transform: Transform,
    ) -> Result<(), String> {
        self.graph.add_node(child_id.clone(), transform);
        self.graph
            .add_edge(parent_id, child_id, ())
            .map_err(|_| "Failed to add edge. Does the parent exist?".to_string())
    }

    fn calculate_global_transforms(&self) -> HashMap<ID, Transform> {
        let mut global_transforms = HashMap::new();

        fn traverse<ID: Clone + Eq + Hash + Ord + Debug>(
            graph: &NodeGraph<ID, (), Transform>,
            node_id: &ID,
            current_transform: Transform,
            global_transforms: &mut HashMap<ID, Transform>,
        ) {
            if let Some(local_transform) = graph.node_data(&node_id.clone()) {
                let global_transform = local_transform.aggregate(&current_transform);
                global_transforms.insert(node_id.clone(), global_transform);

                if let Some(children) = graph.get_edges_connected_to_node(node_id) {
                    for (child_id, _) in children {
                        traverse(graph, &child_id, global_transform, global_transforms);
                    }
                }
            }
        }

        let root_transform = self
            .graph
            .node_data(&self.root_id.clone())
            .unwrap_or(&Transform::new(0.0, 0.0, 0.0))
            .clone();

        traverse(
            &self.graph,
            &self.root_id,
            root_transform,
            &mut global_transforms,
        );

        global_transforms
    }
}

fn main() {
    let mut scenegraph = SceneGraph::new("root".to_string());

    scenegraph
        .add_child(
            "child1".to_string(),
            "root".to_string(),
            Transform::new(1.0, 0.0, 30.0),
        )
        .expect("Failed to add child1");

    scenegraph
        .add_child(
            "child2".to_string(),
            "root".to_string(),
            Transform::new(0.0, 2.0, -15.0),
        )
        .expect("Failed to add child2");

    scenegraph
        .add_child(
            "grandchild".to_string(),
            "child1".to_string(),
            Transform::new(0.5, 0.5, 45.0),
        )
        .expect("Failed to add grandchild");

    let global_transforms = scenegraph.calculate_global_transforms();
    for (id, transform) in global_transforms {
        println!(
            "Global transform for node {:?}: position = {:?}, rotation = {}",
            id, transform.position, transform.rotation
        );
    }
}
