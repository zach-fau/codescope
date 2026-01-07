//! JSON export implementation.
//!
//! Exports dependency analysis results in JSON format for machine-readable output.

use super::{ExportData, Exporter};
use serde::Serialize;
use std::io::{self, Write};

/// JSON exporter implementation.
pub struct JsonExporter;

/// Serializable dependency for JSON output.
#[derive(Serialize)]
struct JsonDependency {
    name: String,
    version: String,
    #[serde(rename = "type")]
    dep_type: String,
}

/// Serializable cycle info for JSON output.
#[derive(Serialize)]
struct JsonCycle {
    packages: Vec<String>,
    path: String,
}

/// Serializable version conflict for JSON output.
#[derive(Serialize)]
struct JsonVersionConflict {
    package: String,
    requirements: Vec<JsonVersionRequirement>,
}

/// Serializable version requirement for JSON output.
#[derive(Serialize)]
struct JsonVersionRequirement {
    version: String,
    required_by: String,
}

/// Summary statistics for JSON output.
#[derive(Serialize)]
struct JsonSummary {
    total_dependencies: usize,
    production: usize,
    development: usize,
    peer: usize,
    optional: usize,
    circular_dependencies: usize,
    version_conflicts: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_bundle_size_bytes: Option<u64>,
}

/// Root JSON export structure.
#[derive(Serialize)]
struct JsonExport {
    project: JsonProject,
    summary: JsonSummary,
    dependencies: Vec<JsonDependency>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    circular_dependencies: Vec<JsonCycle>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    version_conflicts: Vec<JsonVersionConflict>,
}

/// Project info for JSON output.
#[derive(Serialize)]
struct JsonProject {
    name: String,
    version: String,
}

impl Exporter for JsonExporter {
    fn export<W: Write>(&self, data: &ExportData, writer: &mut W) -> io::Result<()> {
        let dependencies: Vec<JsonDependency> = data
            .dependencies
            .iter()
            .map(|d| JsonDependency {
                name: d.name.clone(),
                version: d.version.clone(),
                dep_type: d.dep_type.to_string(),
            })
            .collect();

        let circular_dependencies: Vec<JsonCycle> = data
            .cycles
            .iter()
            .map(|c| JsonCycle {
                packages: c.nodes.clone(),
                path: c.cycle_path(),
            })
            .collect();

        let version_conflicts: Vec<JsonVersionConflict> = data
            .version_conflicts
            .iter()
            .map(|c| JsonVersionConflict {
                package: c.package_name.clone(),
                requirements: c
                    .requirements
                    .iter()
                    .map(|r| JsonVersionRequirement {
                        version: r.version.clone(),
                        required_by: r.required_by.clone(),
                    })
                    .collect(),
            })
            .collect();

        let export = JsonExport {
            project: JsonProject {
                name: data.project_name.clone(),
                version: data.project_version.clone(),
            },
            summary: JsonSummary {
                total_dependencies: data.dependencies.len(),
                production: data.production_count(),
                development: data.dev_count(),
                peer: data.peer_count(),
                optional: data.optional_count(),
                circular_dependencies: data.cycles.len(),
                version_conflicts: data.version_conflicts.len(),
                total_bundle_size_bytes: data.total_bundle_size,
            },
            dependencies,
            circular_dependencies,
            version_conflicts,
        };

        let json = serde_json::to_string_pretty(&export)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        writeln!(writer, "{}", json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DependencyGraph;
    use crate::parser::{Dependency, DependencyType};

    fn create_test_data() -> ExportData {
        let deps = vec![
            Dependency::new("react", "^18.0.0", DependencyType::Production),
            Dependency::new("lodash", "^4.17.21", DependencyType::Production),
            Dependency::new("typescript", "^5.0.0", DependencyType::Development),
        ];

        let graph = DependencyGraph::new();

        ExportData::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            deps,
            &graph,
        )
    }

    #[test]
    fn test_json_export_basic() {
        let data = create_test_data();
        let mut output = Vec::new();

        JsonExporter.export(&data, &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["project"]["name"], "test-project");
        assert_eq!(parsed["project"]["version"], "1.0.0");
        assert_eq!(parsed["summary"]["total_dependencies"], 3);
        assert_eq!(parsed["summary"]["production"], 2);
        assert_eq!(parsed["summary"]["development"], 1);
    }

    #[test]
    fn test_json_export_dependencies_list() {
        let data = create_test_data();
        let mut output = Vec::new();

        JsonExporter.export(&data, &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let deps = parsed["dependencies"].as_array().unwrap();
        assert_eq!(deps.len(), 3);

        // Check first dependency
        assert_eq!(deps[0]["name"], "react");
        assert_eq!(deps[0]["version"], "^18.0.0");
        assert_eq!(deps[0]["type"], "production");
    }

    #[test]
    fn test_json_export_with_cycles() {
        let deps = vec![
            Dependency::new("a", "1.0.0", DependencyType::Production),
            Dependency::new("b", "1.0.0", DependencyType::Production),
        ];

        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", crate::graph::DependencyType::Production);
        graph.add_dependency("b", "1.0.0", crate::graph::DependencyType::Production);
        graph.add_edge("a", "b");
        graph.add_edge("b", "a");

        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        JsonExporter.export(&data, &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(parsed["circular_dependencies"].as_array().unwrap().len() > 0);
        assert_eq!(parsed["summary"]["circular_dependencies"], 1);
    }

    #[test]
    fn test_json_export_with_conflicts() {
        let deps = vec![Dependency::new(
            "lodash",
            "4.17.0",
            DependencyType::Production,
        )];

        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", crate::graph::DependencyType::Production);
        graph.track_version_requirement("lodash", "^4.17.0", "app-a");
        graph.track_version_requirement("lodash", "^4.16.0", "app-b");

        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        JsonExporter.export(&data, &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["version_conflicts"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["version_conflicts"][0]["package"], "lodash");
    }

    #[test]
    fn test_json_is_valid() {
        let data = create_test_data();
        let mut output = Vec::new();

        JsonExporter.export(&data, &mut output).unwrap();

        let json_str = String::from_utf8(output).unwrap();

        // Verify it's valid JSON
        let result: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(result.is_ok());
    }
}
