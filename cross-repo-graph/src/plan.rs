use petgraph::Directed;
use serde::{Deserialize, Serialize};
use crate::error::BuildPlanError;

type GraphIx = u32;
type Node = Repo;
type Edge = ();
type Graph = petgraph::stable_graph::StableGraph<Node, Edge, Directed, GraphIx>;

/// Represents a repository, a node in the dependency graph.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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

/// A build plan is describing dependency relations between different repos.
pub struct BuildPlan {
    pub graph: Graph,
}

impl BuildPlan {
    /// Try to generate a `BuildPlan` from a `Manifest`.
    pub fn try_from_manifest() -> Result<Self, BuildPlanError> {
        todo!()
    }
}
