use std::collections::HashSet;

use super::targets::ProjectTarget;

/// Scratch project.json file definition.
/// Scratch projects are essentially zip archives with all the assets and
/// json definition of the "program" itself.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub meta: ProjectMeta,
    /// Extensions like "pen" that should be loaded in the project.
    pub extensions: HashSet<String>,
    pub monitors: Vec<()>,
    /// Stage, sprites and stuff like that.
    pub targets: Vec<ProjectTarget>,
}

/// Project metadatata. This contains things like Scratch editor version,
/// Scratch engine (vm) version and browser user agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectMeta {
    /// Version of scratch editor used to edit the project.
    pub semver: String,
    /// Version of scratch engine (vm).
    pub vm: String,
    /// User agent used last to edit the project.
    pub agent: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            semver: "3.0.0".to_string(),
            vm: "4.8.75".to_string(),
            agent: "Pawgen/0.1.0-alpha".to_string(),
        }
    }
}
