//! CSV export implementation.
//!
//! Exports dependency analysis results in CSV format for spreadsheet use.

use super::{ExportData, Exporter};
use std::io::{self, Write};

/// CSV exporter implementation.
pub struct CsvExporter;

impl CsvExporter {
    /// Escape a field value for CSV format.
    ///
    /// Wraps the value in quotes if it contains commas, quotes, or newlines.
    fn escape_field(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

impl Exporter for CsvExporter {
    fn export<W: Write>(&self, data: &ExportData, writer: &mut W) -> io::Result<()> {
        // Write header
        writeln!(writer, "name,version,type,in_cycle,has_conflict")?;

        // Build a set of packages in cycles for quick lookup
        let cycle_packages: std::collections::HashSet<&str> = data
            .cycles
            .iter()
            .flat_map(|c| c.nodes.iter().map(|s| s.as_str()))
            .collect();

        // Build a set of packages with conflicts for quick lookup
        let conflict_packages: std::collections::HashSet<&str> = data
            .version_conflicts
            .iter()
            .map(|c| c.package_name.as_str())
            .collect();

        // Write each dependency as a row
        for dep in &data.dependencies {
            let in_cycle = cycle_packages.contains(dep.name.as_str());
            let has_conflict = conflict_packages.contains(dep.name.as_str());

            writeln!(
                writer,
                "{},{},{},{},{}",
                Self::escape_field(&dep.name),
                Self::escape_field(&dep.version),
                dep.dep_type.label(),
                in_cycle,
                has_conflict
            )?;
        }

        Ok(())
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
    fn test_csv_export_basic() {
        let data = create_test_data();
        let mut output = Vec::new();

        CsvExporter.export(&data, &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();

        // Header + 3 dependencies
        assert_eq!(lines.len(), 4);

        // Check header
        assert_eq!(lines[0], "name,version,type,in_cycle,has_conflict");

        // Check first data row
        assert_eq!(lines[1], "react,^18.0.0,prod,false,false");
    }

    #[test]
    fn test_csv_export_all_types() {
        let deps = vec![
            Dependency::new("react", "^18.0.0", DependencyType::Production),
            Dependency::new("typescript", "^5.0.0", DependencyType::Development),
            Dependency::new("react-dom", "^18.0.0", DependencyType::Peer),
            Dependency::new("colors", "^1.0.0", DependencyType::Optional),
        ];

        let graph = DependencyGraph::new();
        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        CsvExporter.export(&data, &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();

        assert!(csv_str.contains(",prod,"));
        assert!(csv_str.contains(",dev,"));
        assert!(csv_str.contains(",peer,"));
        assert!(csv_str.contains(",optional,"));
    }

    #[test]
    fn test_csv_escape_field() {
        // No escaping needed
        assert_eq!(CsvExporter::escape_field("simple"), "simple");

        // Contains comma
        assert_eq!(
            CsvExporter::escape_field("has,comma"),
            "\"has,comma\""
        );

        // Contains quotes
        assert_eq!(
            CsvExporter::escape_field("has\"quote"),
            "\"has\"\"quote\""
        );

        // Contains newline
        assert_eq!(
            CsvExporter::escape_field("has\nnewline"),
            "\"has\nnewline\""
        );
    }

    #[test]
    fn test_csv_export_with_cycles() {
        let deps = vec![
            Dependency::new("a", "1.0.0", DependencyType::Production),
            Dependency::new("b", "1.0.0", DependencyType::Production),
            Dependency::new("c", "1.0.0", DependencyType::Production),
        ];

        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", crate::graph::DependencyType::Production);
        graph.add_dependency("b", "1.0.0", crate::graph::DependencyType::Production);
        graph.add_dependency("c", "1.0.0", crate::graph::DependencyType::Production);
        graph.add_edge("a", "b");
        graph.add_edge("b", "a");

        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        CsvExporter.export(&data, &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();

        // a and b should be in cycle, c should not
        assert!(lines[1].ends_with(",true,false")); // a
        assert!(lines[2].ends_with(",true,false")); // b
        assert!(lines[3].ends_with(",false,false")); // c
    }

    #[test]
    fn test_csv_export_with_conflicts() {
        let deps = vec![
            Dependency::new("lodash", "4.17.0", DependencyType::Production),
            Dependency::new("react", "18.0.0", DependencyType::Production),
        ];

        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", crate::graph::DependencyType::Production);
        graph.add_dependency("react", "18.0.0", crate::graph::DependencyType::Production);
        graph.track_version_requirement("lodash", "^4.17.0", "app-a");
        graph.track_version_requirement("lodash", "^4.16.0", "app-b");

        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        CsvExporter.export(&data, &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv_str.lines().collect();

        // lodash should have conflict, react should not
        assert!(lines[1].ends_with(",false,true")); // lodash
        assert!(lines[2].ends_with(",false,false")); // react
    }

    #[test]
    fn test_csv_export_special_characters() {
        let deps = vec![Dependency::new(
            "@scope/package-name",
            ">=1.0.0, <2.0.0",
            DependencyType::Production,
        )];

        let graph = DependencyGraph::new();
        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        CsvExporter.export(&data, &mut output).unwrap();

        let csv_str = String::from_utf8(output).unwrap();

        // Version with comma should be quoted
        assert!(csv_str.contains("\">=1.0.0, <2.0.0\""));
    }
}
