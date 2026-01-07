//! Export functionality for dependency analysis results.
//!
//! This module provides exporters for outputting dependency analysis
//! results in various formats: JSON, CSV, and Markdown.

pub mod csv;
pub mod json;
pub mod markdown;

use crate::graph::{CycleInfo, DependencyGraph, VersionConflict};
use crate::parser::Dependency;
use std::io::{self, Write};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format - machine-readable, full data
    Json,
    /// CSV format - spreadsheet-friendly
    Csv,
    /// Markdown format - documentation/reporting
    Markdown,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "markdown" | "md" => Ok(ExportFormat::Markdown),
            _ => Err(format!(
                "Unknown export format: '{}'. Valid formats: json, csv, markdown",
                s
            )),
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Data container for export operations.
///
/// Holds all the analysis results that can be exported.
#[derive(Debug, Clone)]
pub struct ExportData {
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// List of all dependencies
    pub dependencies: Vec<Dependency>,
    /// Detected circular dependencies
    pub cycles: Vec<CycleInfo>,
    /// Detected version conflicts
    pub version_conflicts: Vec<VersionConflict>,
    /// Total bundle size (if available)
    pub total_bundle_size: Option<u64>,
}

impl ExportData {
    /// Create new export data from analysis results.
    pub fn new(
        project_name: String,
        project_version: String,
        dependencies: Vec<Dependency>,
        graph: &DependencyGraph,
    ) -> Self {
        Self {
            project_name,
            project_version,
            dependencies,
            cycles: graph.get_cycle_details(),
            version_conflicts: graph.detect_version_conflicts(),
            total_bundle_size: {
                let total = graph.total_bundle_size();
                if total > 0 {
                    Some(total)
                } else {
                    None
                }
            },
        }
    }

    /// Get count of production dependencies
    pub fn production_count(&self) -> usize {
        self.dependencies
            .iter()
            .filter(|d| d.is_production())
            .count()
    }

    /// Get count of development dependencies
    pub fn dev_count(&self) -> usize {
        self.dependencies
            .iter()
            .filter(|d| d.is_development())
            .count()
    }

    /// Get count of peer dependencies
    pub fn peer_count(&self) -> usize {
        use crate::parser::DependencyType;
        self.dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Peer)
            .count()
    }

    /// Get count of optional dependencies
    pub fn optional_count(&self) -> usize {
        use crate::parser::DependencyType;
        self.dependencies
            .iter()
            .filter(|d| d.dep_type == DependencyType::Optional)
            .count()
    }
}

/// Trait for exporters.
pub trait Exporter {
    /// Export the data to the given writer.
    fn export<W: Write>(&self, data: &ExportData, writer: &mut W) -> io::Result<()>;
}

/// Export data in the specified format.
pub fn export<W: Write>(
    format: ExportFormat,
    data: &ExportData,
    writer: &mut W,
) -> io::Result<()> {
    match format {
        ExportFormat::Json => json::JsonExporter.export(data, writer),
        ExportFormat::Csv => csv::CsvExporter.export(data, writer),
        ExportFormat::Markdown => markdown::MarkdownExporter.export(data, writer),
    }
}

/// Export data to a string.
pub fn export_to_string(format: ExportFormat, data: &ExportData) -> io::Result<String> {
    let mut buffer = Vec::new();
    export(format, data, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_str() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("JSON".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("csv".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
        assert_eq!(
            "markdown".parse::<ExportFormat>().unwrap(),
            ExportFormat::Markdown
        );
        assert_eq!(
            "md".parse::<ExportFormat>().unwrap(),
            ExportFormat::Markdown
        );
        assert!("invalid".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::Json), "json");
        assert_eq!(format!("{}", ExportFormat::Csv), "csv");
        assert_eq!(format!("{}", ExportFormat::Markdown), "markdown");
    }
}
