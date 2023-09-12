use std::collections::HashMap;

use crate::{error::BuildPlanError, manifest::Manifest};
use petgraph::Directed;
use serde::{Deserialize, Serialize};

type GraphIx = u32;
type Node = Repo;
type Edge = ();
type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;

type NodeIx = petgraph::prelude::NodeIndex;

/// Represents a repository, a node in the dependency graph.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Repo {
    /// Name of the repository.
    name: String,
    /// Owner of the repository.
    owner: String,
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {} - owner: {}", self.name, self.owner)
    }
}

impl Repo {
    pub fn new(name: String, owner: String) -> Self {
        Self { name, owner }
    }
}

/// A plan is describing dependency relations between different repos.
pub struct Plan {
    graph: Graph,
    repo_to_node: HashMap<Repo, NodeIx>,
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

        Ok(Self {
            graph,
            repo_to_node,
        })
    }

    /// Returns a reference to the stable graph describing the dependency relation.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns the immediate neighbors of the given repo.
    pub fn neighbors(
        &self,
        repo: Repo,
    ) -> Result<impl Iterator<Item = &Repo> + '_, BuildPlanError> {
        let node_ix = self
            .repo_to_node
            .get(&repo)
            .ok_or(BuildPlanError::RepoNotFoundInGraph(repo))?;
        let graph = self.graph();

        Ok(graph
            .neighbors(*node_ix)
            .map(|neighbor_ix| &graph[neighbor_ix]))
    }
}

#[cfg(test)]
mod tests {
    use super::{Plan, Repo};
    use crate::manifest::ManifestFile;

    #[test]
    fn generate_plan_with_two_projects() {
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

    #[test]
    fn test_neighbor_query_with_cycle() {
        let manifest_str = r#"
[project.sway.repo]
name = "sway"
owner = "FuelLabs"

[project.sway]
dependencies = ["rust-sdk"]

[project.rust-sdk.repo]
name = "fuels-rs"
owner = "FuelLabs"

[project.rust-sdk]
dependencies = ["sway"]
"#;
        let manifest = ManifestFile::try_from(manifest_str.to_string())
            .unwrap()
            .manifest();
        let plan = Plan::try_from_manifest(manifest).unwrap();

        let sway_name = "sway".to_string();
        let sway_owner = "FuelLabs".to_string();
        let sway_repo = Repo::new(sway_name, sway_owner);

        let fuels_rs_name = "fuels-rs".to_string();
        let fuels_rs_owner = "FuelLabs".to_string();
        let fuels_rs_repo = Repo::new(fuels_rs_name, fuels_rs_owner);

        let sway_neighbors: Vec<_> = plan
            .neighbors(sway_repo.clone())
            .unwrap()
            .cloned()
            .collect();
        assert_eq!(sway_neighbors.len(), 1);
        let expected_sway_neighbors = [fuels_rs_repo.clone()];
        assert_eq!(sway_neighbors, expected_sway_neighbors);

        let fuels_rs_neighbors: Vec<_> = plan.neighbors(fuels_rs_repo).unwrap().cloned().collect();
        assert_eq!(fuels_rs_neighbors.len(), 1);
        let expected_fuels_rs_neighbors = [sway_repo];
        assert_eq!(fuels_rs_neighbors, expected_fuels_rs_neighbors)
    }

    #[test]
    fn test_neighbor_query_without_cycle() {
        let manifest_str = r#"
[project.sway.repo]
name = "sway"
owner = "FuelLabs"

[project.sway]
dependencies = ["rust-sdk", "wallet"]

[project.rust-sdk.repo]
name = "fuels-rs"
owner = "FuelLabs"

[project.wallet.repo]
name = "forc-wallet"
owner = "FuelLabs"

[project.wallet]
dependencies = ["rust-sdk"]
"#;
        let manifest = ManifestFile::try_from(manifest_str.to_string())
            .unwrap()
            .manifest();
        let plan = Plan::try_from_manifest(manifest).unwrap();

        let sway_name = "sway".to_string();
        let sway_owner = "FuelLabs".to_string();
        let sway_repo = Repo::new(sway_name, sway_owner);

        let fuels_rs_name = "fuels-rs".to_string();
        let fuels_rs_owner = "FuelLabs".to_string();
        let fuels_rs_repo = Repo::new(fuels_rs_name, fuels_rs_owner);

        let forc_wallet_name = "forc-wallet".to_string();
        let forc_wallet_owner = "FuelLabs".to_string();
        let forc_wallet_repo = Repo::new(forc_wallet_name, forc_wallet_owner);

        let mut sway_neighbors: Vec<_> = plan
            .neighbors(sway_repo.clone())
            .unwrap()
            .cloned()
            .collect();
        sway_neighbors.sort();
        assert_eq!(sway_neighbors.len(), 2);
        let mut expected_sway_neighbors = [fuels_rs_repo.clone(), forc_wallet_repo.clone()];
        expected_sway_neighbors.sort();
        assert_eq!(sway_neighbors, expected_sway_neighbors);

        let fuels_rs_neighbors: Vec<_> = plan
            .neighbors(fuels_rs_repo.clone())
            .unwrap()
            .cloned()
            .collect();
        assert_eq!(fuels_rs_neighbors.len(), 0);
        let expected_fuels_rs_neighbors = vec![];
        assert_eq!(fuels_rs_neighbors, expected_fuels_rs_neighbors);

        let forc_wallet_neighbors: Vec<_> =
            plan.neighbors(forc_wallet_repo).unwrap().cloned().collect();
        assert_eq!(forc_wallet_neighbors.len(), 1);
        let expected_forc_wallet_neighbors = vec![fuels_rs_repo];
        assert_eq!(forc_wallet_neighbors, expected_forc_wallet_neighbors)
    }
}