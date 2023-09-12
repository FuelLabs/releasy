use std::collections::HashMap;

use crate::{error::BuildPlanError, manifest::Manifest};
use petgraph::Directed;
use serde::{Deserialize, Serialize};

type GraphIx = u32;
type Node = Repo;
type Edge = ();
type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;

/// Represents a repository, a node in the dependency graph.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Repo {
    /// Name of the repository.
    name: String,
    /// Owner of the repository.
    owner: String,
}

impl Repo {
    pub fn new(name: String, owner: String) -> Self {
        Self { name, owner }
    }
}

/// A plan is describing dependency relations between different repos.
pub struct Plan {
    graph: Graph,
}

impl Plan {
    /// Try to generate a `BuildPlan` from a `Manifest`.
    pub fn try_from_manifest(manifest: Manifest) -> Result<Self, BuildPlanError> {
        let mut graph = Graph::new();
        let project_mapping = manifest.project;

        // Create nodes, for each project in the map create a node and it to the graph.
        // While adding the nodes, keeps a map between node index and the key used to describe that
        // repo from the manifest and repo to the node index.
        let mut key_to_node = HashMap::new();
        let mut repo_to_node = HashMap::new();

        for (key, project) in project_mapping.iter() {
            let node_ix = graph.add_node(project.repo.clone());
            repo_to_node.insert(project.repo.clone(), node_ix);
            key_to_node.insert(key, node_ix);
        }

        // Add edges between nodes with dependency information.
        for project in project_mapping.values() {
            let repo = &project.repo;
            let node_ix_of_current_repo = repo_to_node
                .get(&project.repo)
                .expect("every repo should have a node in the graph!");
            // Collect node indices of dependencies for this project.
            for dependency_key in project.dependencies() {
                let node_ix_of_dependency = key_to_node.get(dependency_key).ok_or_else(|| {
                    BuildPlanError::MissingProjectDefinition(
                        repo.name.clone(),
                        dependency_key.to_string(),
                    )
                })?;

                graph.add_edge(*node_ix_of_current_repo, *node_ix_of_dependency, ());
            }
        }

        Ok(Self { graph })
    }

    /// Returns a reference to the stable graph describing the dependency relation.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::ManifestFile;

    use super::Plan;

    #[test]
    fn generate_build_plan_with_two_projects() {
        let manifest_str = r#"
[project.sway.repo]
name = "sway"
owner = "FuelLabs"

[project.sway]
dependencies = ["rust-sdk"]

[project.rust-sdk.repo]
name = "fuels-rs"
owner = "FuelLabs"
"#;
        let manifest = ManifestFile::try_from(manifest_str.to_string())
            .unwrap()
            .manifest();

        let plan = Plan::try_from_manifest(manifest).unwrap();
        let graph = plan.graph();

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}
