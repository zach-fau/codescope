//! Markdown export implementation.
//!
//! Exports dependency analysis results in Markdown format for documentation and reporting.

use super::{ExportData, Exporter};
use crate::parser::DependencyType;
use crate::ui::format_size;
use std::io::{self, Write};

/// Markdown exporter implementation.
pub struct MarkdownExporter;

impl Exporter for MarkdownExporter {
    fn export<W: Write>(&self, data: &ExportData, writer: &mut W) -> io::Result<()> {
        // Title
        writeln!(writer, "# Dependency Analysis Report")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "**Project:** {} v{}",
            data.project_name, data.project_version
        )?;
        writeln!(writer)?;

        // Summary section
        writeln!(writer, "## Summary")?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| Metric | Count |"
        )?;
        writeln!(writer, "|--------|-------|")?;
        writeln!(
            writer,
            "| Total Dependencies | {} |",
            data.dependencies.len()
        )?;
        writeln!(writer, "| Production | {} |", data.production_count())?;
        writeln!(writer, "| Development | {} |", data.dev_count())?;
        writeln!(writer, "| Peer | {} |", data.peer_count())?;
        writeln!(writer, "| Optional | {} |", data.optional_count())?;
        writeln!(
            writer,
            "| Circular Dependencies | {} |",
            data.cycles.len()
        )?;
        writeln!(
            writer,
            "| Version Conflicts | {} |",
            data.version_conflicts.len()
        )?;

        if let Some(size) = data.total_bundle_size {
            writeln!(writer, "| Total Bundle Size | {} |", format_size(size))?;
        }

        writeln!(writer)?;

        // Dependencies by type
        writeln!(writer, "## Dependencies")?;
        writeln!(writer)?;

        // Production dependencies
        let prod_deps: Vec<_> = data
            .dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Production)
            .collect();

        if !prod_deps.is_empty() {
            writeln!(writer, "### Production Dependencies ({})", prod_deps.len())?;
            writeln!(writer)?;
            writeln!(writer, "| Package | Version |")?;
            writeln!(writer, "|---------|---------|")?;
            for dep in &prod_deps {
                writeln!(writer, "| {} | {} |", dep.name, dep.version)?;
            }
            writeln!(writer)?;
        }

        // Development dependencies
        let dev_deps: Vec<_> = data
            .dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Development)
            .collect();

        if !dev_deps.is_empty() {
            writeln!(
                writer,
                "### Development Dependencies ({})",
                dev_deps.len()
            )?;
            writeln!(writer)?;
            writeln!(writer, "| Package | Version |")?;
            writeln!(writer, "|---------|---------|")?;
            for dep in &dev_deps {
                writeln!(writer, "| {} | {} |", dep.name, dep.version)?;
            }
            writeln!(writer)?;
        }

        // Peer dependencies
        let peer_deps: Vec<_> = data
            .dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Peer)
            .collect();

        if !peer_deps.is_empty() {
            writeln!(writer, "### Peer Dependencies ({})", peer_deps.len())?;
            writeln!(writer)?;
            writeln!(writer, "| Package | Version |")?;
            writeln!(writer, "|---------|---------|")?;
            for dep in &peer_deps {
                writeln!(writer, "| {} | {} |", dep.name, dep.version)?;
            }
            writeln!(writer)?;
        }

        // Optional dependencies
        let optional_deps: Vec<_> = data
            .dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Optional)
            .collect();

        if !optional_deps.is_empty() {
            writeln!(
                writer,
                "### Optional Dependencies ({})",
                optional_deps.len()
            )?;
            writeln!(writer)?;
            writeln!(writer, "| Package | Version |")?;
            writeln!(writer, "|---------|---------|")?;
            for dep in &optional_deps {
                writeln!(writer, "| {} | {} |", dep.name, dep.version)?;
            }
            writeln!(writer)?;
        }

        // Issues section (cycles and conflicts)
        if !data.cycles.is_empty() || !data.version_conflicts.is_empty() {
            writeln!(writer, "## Issues")?;
            writeln!(writer)?;
        }

        // Circular dependencies
        if !data.cycles.is_empty() {
            writeln!(writer, "### Circular Dependencies")?;
            writeln!(writer)?;
            writeln!(
                writer,
                "The following circular dependencies were detected:"
            )?;
            writeln!(writer)?;
            for (i, cycle) in data.cycles.iter().enumerate() {
                writeln!(writer, "{}. `{}`", i + 1, cycle.cycle_path())?;
            }
            writeln!(writer)?;
        }

        // Version conflicts
        if !data.version_conflicts.is_empty() {
            writeln!(writer, "### Version Conflicts")?;
            writeln!(writer)?;
            writeln!(
                writer,
                "The following packages have conflicting version requirements:"
            )?;
            writeln!(writer)?;

            for conflict in &data.version_conflicts {
                writeln!(writer, "#### {}", conflict.package_name)?;
                writeln!(writer)?;
                writeln!(writer, "| Required Version | Required By |")?;
                writeln!(writer, "|------------------|-------------|")?;
                for req in &conflict.requirements {
                    writeln!(writer, "| {} | {} |", req.version, req.required_by)?;
                }
                writeln!(writer)?;
            }
        }

        // Footer
        writeln!(writer, "---")?;
        writeln!(writer, "*Generated by CodeScope*")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DependencyGraph;
    use crate::parser::Dependency;

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
    fn test_markdown_export_basic() {
        let data = create_test_data();
        let mut output = Vec::new();

        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check title
        assert!(md_str.contains("# Dependency Analysis Report"));

        // Check project info
        assert!(md_str.contains("**Project:** test-project v1.0.0"));

        // Check summary table
        assert!(md_str.contains("| Total Dependencies | 3 |"));
        assert!(md_str.contains("| Production | 2 |"));
        assert!(md_str.contains("| Development | 1 |"));
    }

    #[test]
    fn test_markdown_export_sections() {
        let data = create_test_data();
        let mut output = Vec::new();

        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check section headers
        assert!(md_str.contains("## Summary"));
        assert!(md_str.contains("## Dependencies"));
        assert!(md_str.contains("### Production Dependencies (2)"));
        assert!(md_str.contains("### Development Dependencies (1)"));
    }

    #[test]
    fn test_markdown_export_dependency_tables() {
        let data = create_test_data();
        let mut output = Vec::new();

        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check dependency entries
        assert!(md_str.contains("| react | ^18.0.0 |"));
        assert!(md_str.contains("| lodash | ^4.17.21 |"));
        assert!(md_str.contains("| typescript | ^5.0.0 |"));
    }

    #[test]
    fn test_markdown_export_with_cycles() {
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
        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check issues section
        assert!(md_str.contains("## Issues"));
        assert!(md_str.contains("### Circular Dependencies"));
        assert!(md_str.contains("circular dependencies were detected"));
    }

    #[test]
    fn test_markdown_export_with_conflicts() {
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
        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check version conflicts section
        assert!(md_str.contains("### Version Conflicts"));
        assert!(md_str.contains("#### lodash"));
        assert!(md_str.contains("| ^4.17.0 | app-a |"));
        assert!(md_str.contains("| ^4.16.0 | app-b |"));
    }

    #[test]
    fn test_markdown_export_all_types() {
        let deps = vec![
            Dependency::new("react", "^18.0.0", DependencyType::Production),
            Dependency::new("typescript", "^5.0.0", DependencyType::Development),
            Dependency::new("react-dom", "^18.0.0", DependencyType::Peer),
            Dependency::new("colors", "^1.0.0", DependencyType::Optional),
        ];

        let graph = DependencyGraph::new();
        let data = ExportData::new("test".to_string(), "1.0.0".to_string(), deps, &graph);

        let mut output = Vec::new();
        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // All dependency type sections should be present
        assert!(md_str.contains("### Production Dependencies"));
        assert!(md_str.contains("### Development Dependencies"));
        assert!(md_str.contains("### Peer Dependencies"));
        assert!(md_str.contains("### Optional Dependencies"));
    }

    #[test]
    fn test_markdown_export_footer() {
        let data = create_test_data();
        let mut output = Vec::new();

        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Check footer
        assert!(md_str.contains("---"));
        assert!(md_str.contains("*Generated by CodeScope*"));
    }

    #[test]
    fn test_markdown_export_no_issues_section_when_empty() {
        let data = create_test_data();
        let mut output = Vec::new();

        MarkdownExporter.export(&data, &mut output).unwrap();

        let md_str = String::from_utf8(output).unwrap();

        // Issues section should not appear when there are no issues
        assert!(!md_str.contains("## Issues"));
    }
}
