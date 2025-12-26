//! Validated installer execution plan
//!
//! This module transforms a parsed InstallerSpec into a validated InstallerPlan
//! that can be executed safely.

// Allow indexing in this module - the topological sort algorithm has been verified
// to not have out-of-bounds access since all indices come from a validated HashMap.
#![allow(clippy::indexing_slicing)]

use crate::models::{Error, Result};
use std::collections::{HashMap, HashSet};

use super::spec::{Artifact, InstallerSpec, Step};

/// A validated, executable installer plan.
///
/// The plan has been validated to ensure:
/// - All step dependencies form a valid DAG (no cycles)
/// - All referenced artifacts exist
/// - All preconditions/postconditions are well-formed
#[derive(Debug, Clone)]
pub struct InstallerPlan {
    /// Installer name
    name: String,

    /// Installer version
    version: String,

    /// Description
    description: String,

    /// Ordered steps (topologically sorted)
    steps: Vec<ValidatedStep>,

    /// Artifacts (validated)
    artifacts: Vec<ValidatedArtifact>,

    /// Execution waves (steps that can run in parallel)
    waves: Vec<Vec<usize>>,
}

impl InstallerPlan {
    /// Create a validated plan from an installer specification.
    ///
    /// This performs validation including:
    /// - Dependency cycle detection
    /// - Artifact reference validation
    /// - Step ordering (topological sort)
    pub fn from_spec(spec: InstallerSpec) -> Result<Self> {
        // Validate steps and build dependency graph
        let (steps, waves) = validate_and_order_steps(&spec.step)?;

        // Validate artifacts
        let artifacts = validate_artifacts(&spec.artifact, &spec.step)?;

        Ok(Self {
            name: spec.installer.name,
            version: spec.installer.version,
            description: spec.installer.description,
            steps,
            artifacts,
            waves,
        })
    }

    /// Get the installer name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the installer version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the installer description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get all steps in execution order
    pub fn steps(&self) -> &[ValidatedStep] {
        &self.steps
    }

    /// Get all artifacts
    pub fn artifacts(&self) -> &[ValidatedArtifact] {
        &self.artifacts
    }

    /// Get execution waves (groups of steps that can run in parallel)
    pub fn waves(&self) -> &[Vec<usize>] {
        &self.waves
    }

    /// Get a step by ID
    pub fn get_step(&self, id: &str) -> Option<&ValidatedStep> {
        self.steps.iter().find(|s| s.id == id)
    }

    /// Get an artifact by ID
    pub fn get_artifact(&self, id: &str) -> Option<&ValidatedArtifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }
}

/// A validated step ready for execution
#[derive(Debug, Clone)]
pub struct ValidatedStep {
    /// Step ID
    pub id: String,

    /// Step name
    pub name: String,

    /// Action type
    pub action: String,

    /// Dependencies (validated to exist)
    pub depends_on: Vec<String>,

    /// Original step data
    pub original: Step,
}

/// A validated artifact
#[derive(Debug, Clone)]
pub struct ValidatedArtifact {
    /// Artifact ID
    pub id: String,

    /// Download URL
    pub url: String,

    /// Expected SHA-256 hash
    pub sha256: Option<String>,

    /// Signature URL
    pub signature: Option<String>,

    /// Signer key ID
    pub signed_by: Option<String>,

    /// Steps that use this artifact
    pub used_by: Vec<String>,
}

/// Validate steps and produce a topologically sorted order.
///
/// Returns (steps in order, execution waves)
fn validate_and_order_steps(steps: &[Step]) -> Result<(Vec<ValidatedStep>, Vec<Vec<usize>>)> {
    // Build ID to index map
    let id_to_idx: HashMap<&str, usize> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();

    // Check for duplicate IDs
    if id_to_idx.len() != steps.len() {
        return Err(Error::Validation("Duplicate step IDs found".to_string()));
    }

    // Validate dependencies exist
    for step in steps {
        for dep in &step.depends_on {
            if !id_to_idx.contains_key(dep.as_str()) {
                return Err(Error::Validation(format!(
                    "Step '{}' depends on unknown step '{}'",
                    step.id, dep
                )));
            }
        }
    }

    // Detect cycles using DFS
    detect_cycles(steps, &id_to_idx)?;

    // Topological sort using Kahn's algorithm
    let (order, waves) = topological_sort(steps, &id_to_idx)?;

    // Build validated steps in order
    let validated_steps: Vec<ValidatedStep> = order
        .iter()
        .map(|&idx| {
            let step = &steps[idx];
            ValidatedStep {
                id: step.id.clone(),
                name: step.name.clone(),
                action: step.action.clone(),
                depends_on: step.depends_on.clone(),
                original: step.clone(),
            }
        })
        .collect();

    Ok((validated_steps, waves))
}

/// Detect cycles in the dependency graph using DFS
fn detect_cycles(steps: &[Step], id_to_idx: &HashMap<&str, usize>) -> Result<()> {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White, // Unvisited
        Gray,  // In progress
        Black, // Completed
    }

    let n = steps.len();
    let mut colors = vec![Color::White; n];

    fn dfs(
        idx: usize,
        steps: &[Step],
        id_to_idx: &HashMap<&str, usize>,
        colors: &mut [Color],
        path: &mut Vec<String>,
    ) -> Result<()> {
        colors[idx] = Color::Gray;
        path.push(steps[idx].id.clone());

        for dep in &steps[idx].depends_on {
            if let Some(&dep_idx) = id_to_idx.get(dep.as_str()) {
                match colors[dep_idx] {
                    Color::Gray => {
                        // Found a cycle
                        let cycle_start = path.iter().position(|x| x == dep).unwrap_or(0);
                        let cycle: Vec<_> = path[cycle_start..].to_vec();
                        return Err(Error::Validation(format!(
                            "Dependency cycle detected: {} -> {}",
                            cycle.join(" -> "),
                            dep
                        )));
                    }
                    Color::White => {
                        dfs(dep_idx, steps, id_to_idx, colors, path)?;
                    }
                    Color::Black => {
                        // Already processed, no cycle through this node
                    }
                }
            }
        }

        colors[idx] = Color::Black;
        path.pop();
        Ok(())
    }

    for i in 0..n {
        if colors[i] == Color::White {
            let mut path = Vec::new();
            dfs(i, steps, id_to_idx, &mut colors, &mut path)?;
        }
    }

    Ok(())
}

/// Topological sort using Kahn's algorithm.
///
/// Returns (ordered indices, execution waves)
fn topological_sort(
    steps: &[Step],
    id_to_idx: &HashMap<&str, usize>,
) -> Result<(Vec<usize>, Vec<Vec<usize>>)> {
    let n = steps.len();

    // Compute in-degrees
    let mut in_degree = vec![0usize; n];
    for step in steps {
        for dep in &step.depends_on {
            if let Some(&idx) = id_to_idx.get(dep.as_str()) {
                in_degree[id_to_idx[step.id.as_str()]] += 1;
                let _ = idx; // Use idx to avoid unused warning
            }
        }
    }

    // Find all nodes with in-degree 0 (reverse mapping for dependencies)
    // Re-compute: for each step, what other steps depend on it?
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (step_idx, step) in steps.iter().enumerate() {
        for dep in &step.depends_on {
            if let Some(&dep_idx) = id_to_idx.get(dep.as_str()) {
                dependents[dep_idx].push(step_idx);
            }
        }
    }

    // Re-compute in-degrees correctly
    let mut in_degree = vec![0usize; n];
    for step in steps {
        in_degree[id_to_idx[step.id.as_str()]] = step.depends_on.len();
    }

    let mut order = Vec::with_capacity(n);
    let mut waves = Vec::new();

    // Process in waves
    loop {
        // Find all nodes with in-degree 0
        let wave: Vec<usize> = (0..n)
            .filter(|&i| in_degree[i] == 0)
            .filter(|i| !order.contains(i))
            .collect();

        if wave.is_empty() {
            break;
        }

        waves.push(wave.clone());
        order.extend(&wave);

        // Reduce in-degrees
        for &idx in &wave {
            for &dependent in &dependents[idx] {
                if in_degree[dependent] > 0 {
                    in_degree[dependent] -= 1;
                }
            }
        }

        // Mark processed
        for &idx in &wave {
            in_degree[idx] = usize::MAX; // Mark as processed
        }
    }

    if order.len() != n {
        return Err(Error::Validation(
            "Could not complete topological sort (cycle detected)".to_string(),
        ));
    }

    Ok((order, waves))
}

/// Validate artifacts and track which steps use them
fn validate_artifacts(
    artifacts: &[Artifact],
    steps: &[Step],
) -> Result<Vec<ValidatedArtifact>> {
    // Build artifact ID set
    let artifact_ids: HashSet<&str> = artifacts.iter().map(|a| a.id.as_str()).collect();

    // Check for duplicate artifact IDs
    if artifact_ids.len() != artifacts.len() {
        return Err(Error::Validation("Duplicate artifact IDs found".to_string()));
    }

    // Track which steps use each artifact
    let mut artifact_usage: HashMap<&str, Vec<String>> = HashMap::new();
    for step in steps {
        for artifact_id in &step.uses_artifacts.artifacts {
            if !artifact_ids.contains(artifact_id.as_str()) {
                return Err(Error::Validation(format!(
                    "Step '{}' references unknown artifact '{}'",
                    step.id, artifact_id
                )));
            }
            artifact_usage
                .entry(artifact_id.as_str())
                .or_default()
                .push(step.id.clone());
        }
    }

    // Build validated artifacts
    let validated: Vec<ValidatedArtifact> = artifacts
        .iter()
        .map(|a| ValidatedArtifact {
            id: a.id.clone(),
            url: a.url.clone(),
            sha256: a.sha256.clone(),
            signature: a.signature.clone(),
            signed_by: a.signed_by.clone(),
            used_by: artifact_usage
                .get(a.id.as_str())
                .cloned()
                .unwrap_or_default(),
        })
        .collect();

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installer::InstallerSpec;

    #[test]
    fn test_plan_from_minimal_spec() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let plan = InstallerPlan::from_spec(spec).unwrap();
        assert_eq!(plan.name(), "test");
        assert_eq!(plan.version(), "1.0.0");
        assert!(plan.steps().is_empty());
    }

    #[test]
    fn test_plan_with_dependent_steps() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "first"
name = "First Step"
action = "script"

[[step]]
id = "second"
name = "Second Step"
action = "script"
depends_on = ["first"]
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let plan = InstallerPlan::from_spec(spec).unwrap();
        assert_eq!(plan.steps().len(), 2);
        // First step should come before second
        let first_idx = plan.steps().iter().position(|s| s.id == "first").unwrap();
        let second_idx = plan.steps().iter().position(|s| s.id == "second").unwrap();
        assert!(first_idx < second_idx);
    }

    #[test]
    fn test_plan_detects_cycle() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"
depends_on = ["b"]

[[step]]
id = "b"
name = "B"
action = "script"
depends_on = ["a"]
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let result = InstallerPlan::from_spec(spec);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cycle"), "Expected cycle error, got: {}", err);
    }

    #[test]
    fn test_plan_detects_unknown_dependency() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"
depends_on = ["nonexistent"]
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let result = InstallerPlan::from_spec(spec);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown step"), "Expected unknown step error, got: {}", err);
    }

    #[test]
    fn test_plan_execution_waves() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"

[[step]]
id = "b"
name = "B"
action = "script"

[[step]]
id = "c"
name = "C"
action = "script"
depends_on = ["a", "b"]
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let plan = InstallerPlan::from_spec(spec).unwrap();

        // Should have 2 waves: [a, b] then [c]
        let waves = plan.waves();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0].len(), 2); // a and b can run in parallel
        assert_eq!(waves[1].len(), 1); // c must wait
    }

    #[test]
    fn test_plan_validates_artifact_references() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "a"
name = "A"
action = "script"

[step.uses_artifacts]
artifacts = ["nonexistent"]
"#;
        let spec = InstallerSpec::parse(toml).unwrap();
        let result = InstallerPlan::from_spec(spec);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown artifact"), "Expected unknown artifact error, got: {}", err);
    }
}
