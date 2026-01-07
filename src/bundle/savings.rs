//! Bundle size savings calculation module
//!
//! This module calculates potential bundle size savings based on:
//! - Unused or underutilized dependencies
//! - Tree-shaking opportunities based on import utilization
//! - Suggestions for lighter alternatives
//!
//! # Example
//!
//! ```ignore
//! use codescope::bundle::savings::{SavingsCalculator, SavingsReport};
//! use codescope::analysis::ProjectImports;
//! use codescope::bundle::BundleAnalysis;
//!
//! let calculator = SavingsCalculator::new();
//! let report = calculator.calculate(&bundle_analysis, &project_imports, &export_counts);
//!
//! println!("Total potential savings: {}", report.format_total_savings());
//! for saving in report.savings_by_size() {
//!     println!("{}: {} ({:.1}%)", saving.package_name, saving.format_potential_savings(), saving.savings_percentage);
//! }
//! ```

use std::collections::HashMap;

use crate::analysis::exports::{PackageUsage, ProjectImports};
use crate::bundle::webpack::{format_size, BundleAnalysis, PackageBundleSize};

/// Threshold for considering a package as "underutilized"
/// Packages using less than this percentage of their exports may be candidates for optimization
const UNDERUTILIZATION_THRESHOLD: f64 = 20.0;

/// Threshold for considering a package as "unused"
/// Packages with 0% utilization and only side-effect imports
const UNUSED_THRESHOLD: f64 = 1.0;

/// Category of potential savings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavingsCategory {
    /// Package appears to be completely unused (not imported anywhere)
    Unused,
    /// Package is imported but only a small percentage of exports are used
    Underutilized,
    /// Package could benefit from tree-shaking (partial usage)
    TreeShaking,
    /// Package has a lighter alternative available
    HasAlternative,
}

impl SavingsCategory {
    /// Get a display label for the category
    pub fn label(&self) -> &'static str {
        match self {
            SavingsCategory::Unused => "Unused",
            SavingsCategory::Underutilized => "Underutilized",
            SavingsCategory::TreeShaking => "Tree-shaking",
            SavingsCategory::HasAlternative => "Alternative available",
        }
    }

    /// Get a detailed description for the category
    pub fn description(&self) -> &'static str {
        match self {
            SavingsCategory::Unused => "Package is in dependencies but not imported in source code",
            SavingsCategory::Underutilized => "Package is used but most of its exports are unused",
            SavingsCategory::TreeShaking => "Package could have smaller footprint with better tree-shaking",
            SavingsCategory::HasAlternative => "A lighter alternative package exists",
        }
    }
}

/// Represents potential savings for a single package
#[derive(Debug, Clone)]
pub struct PackageSavings {
    /// Package name
    pub package_name: String,
    /// Current bundle size in bytes
    pub current_size: u64,
    /// Estimated potential savings in bytes
    pub potential_savings: u64,
    /// Category of savings
    pub category: SavingsCategory,
    /// Utilization percentage (0-100)
    pub utilization_percentage: Option<f64>,
    /// Number of exports used
    pub exports_used: usize,
    /// Total exports available (if known)
    pub total_exports: Option<usize>,
    /// Suggested action
    pub suggestion: String,
    /// Alternative package suggestion (if applicable)
    pub alternative: Option<String>,
}

impl PackageSavings {
    /// Format the potential savings as a human-readable string
    pub fn format_potential_savings(&self) -> String {
        format_size(self.potential_savings)
    }

    /// Format the current size as a human-readable string
    pub fn format_current_size(&self) -> String {
        format_size(self.current_size)
    }

    /// Calculate the savings as a percentage of current size
    pub fn savings_percentage(&self) -> f64 {
        if self.current_size == 0 {
            0.0
        } else {
            (self.potential_savings as f64 / self.current_size as f64) * 100.0
        }
    }
}

/// Summary of savings report
#[derive(Debug, Clone, Default)]
pub struct SavingsSummary {
    /// Total potential savings across all packages
    pub total_potential_savings: u64,
    /// Total current bundle size
    pub total_bundle_size: u64,
    /// Number of packages with potential savings
    pub packages_with_savings: usize,
    /// Number of unused packages
    pub unused_count: usize,
    /// Number of underutilized packages
    pub underutilized_count: usize,
    /// Number of packages with tree-shaking opportunities
    pub tree_shaking_count: usize,
}

impl SavingsSummary {
    /// Calculate the total savings percentage
    pub fn savings_percentage(&self) -> f64 {
        if self.total_bundle_size == 0 {
            0.0
        } else {
            (self.total_potential_savings as f64 / self.total_bundle_size as f64) * 100.0
        }
    }

    /// Format total potential savings as human-readable string
    pub fn format_total_savings(&self) -> String {
        format_size(self.total_potential_savings)
    }

    /// Format total bundle size as human-readable string
    pub fn format_total_bundle_size(&self) -> String {
        format_size(self.total_bundle_size)
    }
}

/// Complete savings report
#[derive(Debug, Clone, Default)]
pub struct SavingsReport {
    /// Individual package savings
    pub package_savings: Vec<PackageSavings>,
    /// Summary statistics
    pub summary: SavingsSummary,
}

impl SavingsReport {
    /// Get package savings sorted by potential savings (largest first)
    pub fn savings_by_size(&self) -> Vec<&PackageSavings> {
        let mut sorted: Vec<_> = self.package_savings.iter().collect();
        sorted.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));
        sorted
    }

    /// Get package savings filtered by category
    pub fn savings_by_category(&self, category: SavingsCategory) -> Vec<&PackageSavings> {
        self.package_savings
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Format the report as a text string suitable for CI output
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Bundle Size Savings Report ===\n\n");

        // Summary
        output.push_str(&format!(
            "Total Bundle Size: {}\n",
            self.summary.format_total_bundle_size()
        ));
        output.push_str(&format!(
            "Potential Savings: {} ({:.1}%)\n",
            self.summary.format_total_savings(),
            self.summary.savings_percentage()
        ));
        output.push_str(&format!(
            "Packages with savings: {}\n\n",
            self.summary.packages_with_savings
        ));

        // Breakdown by category
        if self.summary.unused_count > 0 {
            output.push_str(&format!("Unused packages: {}\n", self.summary.unused_count));
        }
        if self.summary.underutilized_count > 0 {
            output.push_str(&format!(
                "Underutilized packages: {}\n",
                self.summary.underutilized_count
            ));
        }
        if self.summary.tree_shaking_count > 0 {
            output.push_str(&format!(
                "Tree-shaking opportunities: {}\n",
                self.summary.tree_shaking_count
            ));
        }

        output.push('\n');

        // Individual packages
        if !self.package_savings.is_empty() {
            output.push_str("--- Package Details ---\n\n");

            for saving in self.savings_by_size() {
                output.push_str(&format!(
                    "{} [{}]\n",
                    saving.package_name,
                    saving.category.label()
                ));
                output.push_str(&format!(
                    "  Current size: {}\n",
                    saving.format_current_size()
                ));
                output.push_str(&format!(
                    "  Potential savings: {} ({:.1}%)\n",
                    saving.format_potential_savings(),
                    saving.savings_percentage()
                ));
                if let Some(util) = saving.utilization_percentage {
                    output.push_str(&format!("  Utilization: {:.1}%\n", util));
                }
                output.push_str(&format!("  Suggestion: {}\n", saving.suggestion));
                if let Some(ref alt) = saving.alternative {
                    output.push_str(&format!("  Alternative: {}\n", alt));
                }
                output.push('\n');
            }
        }

        output
    }

    /// Check if there are any savings to report
    pub fn has_savings(&self) -> bool {
        self.summary.total_potential_savings > 0
    }
}

/// Known heavy packages with lighter alternatives
fn get_known_alternatives() -> HashMap<&'static str, (&'static str, &'static str)> {
    let mut alternatives = HashMap::new();

    // (heavy_package, (alternative, description))
    alternatives.insert("moment", ("dayjs", "Day.js is 2KB vs Moment's 67KB"));
    alternatives.insert("lodash", ("lodash-es", "Use lodash-es for better tree-shaking, or individual imports"));
    alternatives.insert("underscore", ("lodash-es", "Lodash-es with tree-shaking is more efficient"));
    alternatives.insert("axios", ("fetch", "Native fetch is built-in and zero-cost"));
    alternatives.insert("request", ("node-fetch", "request is deprecated; use node-fetch or native fetch"));
    alternatives.insert("uuid", ("crypto.randomUUID", "Native crypto.randomUUID() works in modern environments"));
    alternatives.insert("bluebird", ("native Promise", "Native Promises are now performant enough for most cases"));
    alternatives.insert("jquery", ("vanilla JS", "Modern DOM APIs often eliminate the need for jQuery"));

    alternatives
}

/// Calculator for bundle size savings
#[derive(Debug, Default)]
pub struct SavingsCalculator {
    /// Known alternatives for heavy packages
    alternatives: HashMap<String, (String, String)>,
}

impl SavingsCalculator {
    /// Create a new savings calculator
    pub fn new() -> Self {
        let known = get_known_alternatives();
        let alternatives: HashMap<String, (String, String)> = known
            .into_iter()
            .map(|(k, (alt, desc))| (k.to_string(), (alt.to_string(), desc.to_string())))
            .collect();

        Self { alternatives }
    }

    /// Calculate potential savings based on bundle analysis and import usage
    ///
    /// # Arguments
    ///
    /// * `bundle_analysis` - Bundle size information from webpack stats
    /// * `project_imports` - Import usage information from source analysis
    /// * `export_counts` - Map of package names to their total export count
    ///
    /// # Returns
    ///
    /// A `SavingsReport` containing all potential savings opportunities
    pub fn calculate(
        &self,
        bundle_analysis: &BundleAnalysis,
        project_imports: &ProjectImports,
        export_counts: &HashMap<String, usize>,
    ) -> SavingsReport {
        let mut report = SavingsReport::default();

        // Track total bundle size
        report.summary.total_bundle_size = bundle_analysis.total_module_size;

        // Analyze each package in the bundle
        for (package_name, pkg_size) in &bundle_analysis.package_sizes {
            if let Some(saving) = self.analyze_package(
                package_name,
                pkg_size,
                project_imports.package_usage.get(package_name),
                export_counts.get(package_name).copied(),
            ) {
                // Update summary counts
                match saving.category {
                    SavingsCategory::Unused => report.summary.unused_count += 1,
                    SavingsCategory::Underutilized => report.summary.underutilized_count += 1,
                    SavingsCategory::TreeShaking => report.summary.tree_shaking_count += 1,
                    SavingsCategory::HasAlternative => {}
                }

                report.summary.total_potential_savings += saving.potential_savings;
                report.summary.packages_with_savings += 1;
                report.package_savings.push(saving);
            }
        }

        report
    }

    /// Analyze a single package for potential savings
    fn analyze_package(
        &self,
        package_name: &str,
        pkg_size: &PackageBundleSize,
        usage: Option<&PackageUsage>,
        total_exports: Option<usize>,
    ) -> Option<PackageSavings> {
        let current_size = pkg_size.total_size;

        // Check for known alternatives first
        if let Some((alt_name, alt_desc)) = self.alternatives.get(package_name) {
            // For packages with known alternatives, suggest the alternative
            // Estimate 50-90% savings depending on the alternative
            let estimated_savings = match package_name {
                "moment" => (current_size as f64 * 0.97) as u64, // dayjs is ~97% smaller
                "lodash" => (current_size as f64 * 0.70) as u64, // lodash-es with tree-shaking
                "jquery" => (current_size as f64 * 0.90) as u64, // vanilla JS
                _ => (current_size as f64 * 0.50) as u64, // default 50% estimate
            };

            return Some(PackageSavings {
                package_name: package_name.to_string(),
                current_size,
                potential_savings: estimated_savings,
                category: SavingsCategory::HasAlternative,
                utilization_percentage: usage.and_then(|u| u.utilization_percentage(total_exports.unwrap_or(0))),
                exports_used: usage.map(|u| u.export_count()).unwrap_or(0),
                total_exports,
                suggestion: format!("Consider replacing with {}", alt_name),
                alternative: Some(format!("{}: {}", alt_name, alt_desc)),
            });
        }

        // Check usage patterns
        match usage {
            None => {
                // Package is in bundle but not imported anywhere - likely unused
                Some(PackageSavings {
                    package_name: package_name.to_string(),
                    current_size,
                    potential_savings: current_size, // 100% savings if removed
                    category: SavingsCategory::Unused,
                    utilization_percentage: Some(0.0),
                    exports_used: 0,
                    total_exports,
                    suggestion: "Consider removing this unused dependency".to_string(),
                    alternative: None,
                })
            }
            Some(pkg_usage) => {
                let exports_used = pkg_usage.export_count();
                let utilization = pkg_usage.utilization_percentage(total_exports.unwrap_or(0));

                // If namespace import or side-effect only, can't estimate savings
                if pkg_usage.uses_namespace {
                    return None; // Uses all exports
                }

                if pkg_usage.has_side_effects && exports_used == 0 {
                    // Side-effect only import - might be necessary
                    return None;
                }

                match utilization {
                    Some(util) if util < UNUSED_THRESHOLD => {
                        // Almost no exports used
                        Some(PackageSavings {
                            package_name: package_name.to_string(),
                            current_size,
                            potential_savings: (current_size as f64 * 0.95) as u64, // 95% savings
                            category: SavingsCategory::Underutilized,
                            utilization_percentage: Some(util),
                            exports_used,
                            total_exports,
                            suggestion: "Very low utilization - consider removing or finding a smaller alternative".to_string(),
                            alternative: None,
                        })
                    }
                    Some(util) if util < UNDERUTILIZATION_THRESHOLD => {
                        // Underutilized - calculate savings based on unused portion
                        let unused_portion = (100.0 - util) / 100.0;
                        let potential_savings = (current_size as f64 * unused_portion * 0.8) as u64; // 80% of unused portion

                        Some(PackageSavings {
                            package_name: package_name.to_string(),
                            current_size,
                            potential_savings,
                            category: SavingsCategory::Underutilized,
                            utilization_percentage: Some(util),
                            exports_used,
                            total_exports,
                            suggestion: format!(
                                "Only {:.1}% of exports used - consider modular imports or tree-shaking",
                                util
                            ),
                            alternative: None,
                        })
                    }
                    Some(util) if util < 80.0 => {
                        // Moderate usage - tree-shaking opportunity
                        let unused_portion = (100.0 - util) / 100.0;
                        let potential_savings = (current_size as f64 * unused_portion * 0.6) as u64; // 60% of unused

                        // Only report if savings are significant (> 10KB)
                        if potential_savings < 10 * 1024 {
                            return None;
                        }

                        Some(PackageSavings {
                            package_name: package_name.to_string(),
                            current_size,
                            potential_savings,
                            category: SavingsCategory::TreeShaking,
                            utilization_percentage: Some(util),
                            exports_used,
                            total_exports,
                            suggestion: "Good tree-shaking candidate - ensure bundler is configured for tree-shaking".to_string(),
                            alternative: None,
                        })
                    }
                    _ => None, // Well-utilized package
                }
            }
        }
    }

    /// Calculate savings from a simplified input (just package sizes and utilization)
    ///
    /// This is a convenience method for when you have utilization percentages directly.
    pub fn calculate_from_utilization(
        &self,
        package_sizes: &HashMap<String, u64>,
        utilization: &HashMap<String, f64>,
    ) -> SavingsReport {
        let mut report = SavingsReport::default();

        for (package_name, &size) in package_sizes {
            report.summary.total_bundle_size += size;

            let util = utilization.get(package_name).copied();

            if let Some(saving) = self.analyze_from_utilization(package_name, size, util) {
                match saving.category {
                    SavingsCategory::Unused => report.summary.unused_count += 1,
                    SavingsCategory::Underutilized => report.summary.underutilized_count += 1,
                    SavingsCategory::TreeShaking => report.summary.tree_shaking_count += 1,
                    SavingsCategory::HasAlternative => {}
                }

                report.summary.total_potential_savings += saving.potential_savings;
                report.summary.packages_with_savings += 1;
                report.package_savings.push(saving);
            }
        }

        report
    }

    /// Analyze a package given its size and utilization percentage
    fn analyze_from_utilization(
        &self,
        package_name: &str,
        current_size: u64,
        utilization: Option<f64>,
    ) -> Option<PackageSavings> {
        // Check for known alternatives first
        if let Some((alt_name, alt_desc)) = self.alternatives.get(package_name) {
            let estimated_savings = match package_name {
                "moment" => (current_size as f64 * 0.97) as u64,
                "lodash" => (current_size as f64 * 0.70) as u64,
                "jquery" => (current_size as f64 * 0.90) as u64,
                _ => (current_size as f64 * 0.50) as u64,
            };

            return Some(PackageSavings {
                package_name: package_name.to_string(),
                current_size,
                potential_savings: estimated_savings,
                category: SavingsCategory::HasAlternative,
                utilization_percentage: utilization,
                exports_used: 0,
                total_exports: None,
                suggestion: format!("Consider replacing with {}", alt_name),
                alternative: Some(format!("{}: {}", alt_name, alt_desc)),
            });
        }

        match utilization {
            None => {
                // Unknown utilization - assume unused
                Some(PackageSavings {
                    package_name: package_name.to_string(),
                    current_size,
                    potential_savings: current_size,
                    category: SavingsCategory::Unused,
                    utilization_percentage: None,
                    exports_used: 0,
                    total_exports: None,
                    suggestion: "Consider removing this unused dependency".to_string(),
                    alternative: None,
                })
            }
            Some(util) if util < UNUSED_THRESHOLD => {
                Some(PackageSavings {
                    package_name: package_name.to_string(),
                    current_size,
                    potential_savings: (current_size as f64 * 0.95) as u64,
                    category: SavingsCategory::Underutilized,
                    utilization_percentage: Some(util),
                    exports_used: 0,
                    total_exports: None,
                    suggestion: "Very low utilization - consider removing".to_string(),
                    alternative: None,
                })
            }
            Some(util) if util < UNDERUTILIZATION_THRESHOLD => {
                let unused_portion = (100.0 - util) / 100.0;
                let potential_savings = (current_size as f64 * unused_portion * 0.8) as u64;

                Some(PackageSavings {
                    package_name: package_name.to_string(),
                    current_size,
                    potential_savings,
                    category: SavingsCategory::Underutilized,
                    utilization_percentage: Some(util),
                    exports_used: 0,
                    total_exports: None,
                    suggestion: format!("Only {:.1}% utilized - consider modular imports", util),
                    alternative: None,
                })
            }
            Some(util) if util < 80.0 => {
                let unused_portion = (100.0 - util) / 100.0;
                let potential_savings = (current_size as f64 * unused_portion * 0.6) as u64;

                if potential_savings < 10 * 1024 {
                    return None;
                }

                Some(PackageSavings {
                    package_name: package_name.to_string(),
                    current_size,
                    potential_savings,
                    category: SavingsCategory::TreeShaking,
                    utilization_percentage: Some(util),
                    exports_used: 0,
                    total_exports: None,
                    suggestion: "Tree-shaking opportunity".to_string(),
                    alternative: None,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::webpack::PackageBundleSize;

    fn create_test_bundle_analysis() -> BundleAnalysis {
        let mut analysis = BundleAnalysis::default();

        // Add some test packages
        let mut lodash = PackageBundleSize::new("lodash");
        lodash.add_module("lodash/index.js".to_string(), 70 * 1024); // 70KB
        analysis.package_sizes.insert("lodash".to_string(), lodash);

        let mut moment = PackageBundleSize::new("moment");
        moment.add_module("moment/moment.js".to_string(), 300 * 1024); // 300KB
        analysis.package_sizes.insert("moment".to_string(), moment);

        let mut react = PackageBundleSize::new("react");
        react.add_module("react/index.js".to_string(), 50 * 1024); // 50KB
        analysis.package_sizes.insert("react".to_string(), react);

        let mut unused_pkg = PackageBundleSize::new("unused-pkg");
        unused_pkg.add_module("unused-pkg/index.js".to_string(), 20 * 1024); // 20KB
        analysis.package_sizes.insert("unused-pkg".to_string(), unused_pkg);

        analysis.total_module_size = 440 * 1024;

        analysis
    }

    fn create_test_project_imports() -> ProjectImports {
        let mut imports = ProjectImports::new();

        // React is well-used
        let mut react_usage = PackageUsage::default();
        react_usage.named_imports.insert("useState".to_string());
        react_usage.named_imports.insert("useEffect".to_string());
        react_usage.named_imports.insert("useCallback".to_string());
        react_usage.uses_default = true;
        react_usage.importing_files.insert("src/App.tsx".to_string());
        imports.package_usage.insert("react".to_string(), react_usage);

        // Lodash is underutilized
        let mut lodash_usage = PackageUsage::default();
        lodash_usage.named_imports.insert("debounce".to_string());
        lodash_usage.importing_files.insert("src/utils.ts".to_string());
        imports.package_usage.insert("lodash".to_string(), lodash_usage);

        // Moment is imported
        let mut moment_usage = PackageUsage::default();
        moment_usage.uses_default = true;
        moment_usage.importing_files.insert("src/date.ts".to_string());
        imports.package_usage.insert("moment".to_string(), moment_usage);

        // unused-pkg is NOT in imports (not imported anywhere)

        imports
    }

    fn create_test_export_counts() -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        counts.insert("react".to_string(), 20);
        counts.insert("lodash".to_string(), 300);
        counts.insert("moment".to_string(), 50);
        counts.insert("unused-pkg".to_string(), 10);
        counts
    }

    #[test]
    fn test_savings_category_label() {
        assert_eq!(SavingsCategory::Unused.label(), "Unused");
        assert_eq!(SavingsCategory::Underutilized.label(), "Underutilized");
        assert_eq!(SavingsCategory::TreeShaking.label(), "Tree-shaking");
        assert_eq!(SavingsCategory::HasAlternative.label(), "Alternative available");
    }

    #[test]
    fn test_savings_category_description() {
        assert!(!SavingsCategory::Unused.description().is_empty());
        assert!(!SavingsCategory::Underutilized.description().is_empty());
    }

    #[test]
    fn test_package_savings_formatting() {
        let saving = PackageSavings {
            package_name: "lodash".to_string(),
            current_size: 100 * 1024, // 100KB
            potential_savings: 70 * 1024, // 70KB
            category: SavingsCategory::Underutilized,
            utilization_percentage: Some(5.0),
            exports_used: 1,
            total_exports: Some(300),
            suggestion: "Test suggestion".to_string(),
            alternative: None,
        };

        assert_eq!(saving.format_current_size(), "100.00 KB");
        assert_eq!(saving.format_potential_savings(), "70.00 KB");
        assert!((saving.savings_percentage() - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_package_savings_zero_size() {
        let saving = PackageSavings {
            package_name: "empty".to_string(),
            current_size: 0,
            potential_savings: 0,
            category: SavingsCategory::Unused,
            utilization_percentage: None,
            exports_used: 0,
            total_exports: None,
            suggestion: "".to_string(),
            alternative: None,
        };

        assert_eq!(saving.savings_percentage(), 0.0);
    }

    #[test]
    fn test_savings_summary_percentage() {
        let summary = SavingsSummary {
            total_potential_savings: 50 * 1024,
            total_bundle_size: 200 * 1024,
            packages_with_savings: 2,
            unused_count: 1,
            underutilized_count: 1,
            tree_shaking_count: 0,
        };

        assert!((summary.savings_percentage() - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_savings_summary_zero_bundle() {
        let summary = SavingsSummary {
            total_bundle_size: 0,
            ..Default::default()
        };

        assert_eq!(summary.savings_percentage(), 0.0);
    }

    #[test]
    fn test_savings_calculator_new() {
        let calc = SavingsCalculator::new();
        assert!(calc.alternatives.contains_key("moment"));
        assert!(calc.alternatives.contains_key("lodash"));
    }

    #[test]
    fn test_calculator_detects_unused_package() {
        let calc = SavingsCalculator::new();
        let bundle = create_test_bundle_analysis();
        let imports = create_test_project_imports();
        let exports = create_test_export_counts();

        let report = calc.calculate(&bundle, &imports, &exports);

        // unused-pkg should be detected as unused
        let unused = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "unused-pkg");
        assert!(unused.is_some());
        assert_eq!(unused.unwrap().category, SavingsCategory::Unused);
    }

    #[test]
    fn test_calculator_detects_underutilized_package() {
        let calc = SavingsCalculator::new();
        let bundle = create_test_bundle_analysis();
        let imports = create_test_project_imports();
        let exports = create_test_export_counts();

        let report = calc.calculate(&bundle, &imports, &exports);

        // lodash should be detected as underutilized (1 out of 300 exports)
        let lodash = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "lodash");
        assert!(lodash.is_some());
        // Could be HasAlternative or Underutilized depending on logic priority
        assert!(matches!(
            lodash.unwrap().category,
            SavingsCategory::Underutilized | SavingsCategory::HasAlternative
        ));
    }

    #[test]
    fn test_calculator_suggests_alternatives() {
        let calc = SavingsCalculator::new();
        let bundle = create_test_bundle_analysis();
        let imports = create_test_project_imports();
        let exports = create_test_export_counts();

        let report = calc.calculate(&bundle, &imports, &exports);

        // moment should have alternative suggestion
        let moment = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "moment");
        assert!(moment.is_some());
        assert_eq!(moment.unwrap().category, SavingsCategory::HasAlternative);
        assert!(moment.unwrap().alternative.is_some());
    }

    #[test]
    fn test_calculator_well_utilized_not_reported() {
        let calc = SavingsCalculator::new();

        // Create a bundle with well-utilized package
        let mut analysis = BundleAnalysis::default();
        let mut well_used = PackageBundleSize::new("well-used");
        well_used.add_module("well-used/index.js".to_string(), 50 * 1024);
        analysis.package_sizes.insert("well-used".to_string(), well_used);
        analysis.total_module_size = 50 * 1024;

        // Create imports with high utilization
        let mut imports = ProjectImports::new();
        let mut usage = PackageUsage::default();
        for i in 0..9 {
            usage.named_imports.insert(format!("export{}", i));
        }
        usage.uses_default = true;
        usage.importing_files.insert("src/app.ts".to_string());
        imports.package_usage.insert("well-used".to_string(), usage);

        let mut exports = HashMap::new();
        exports.insert("well-used".to_string(), 10); // 10 out of 10 = 100%

        let report = calc.calculate(&analysis, &imports, &exports);

        // well-used should NOT be in the report
        let found = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "well-used");
        assert!(found.is_none());
    }

    #[test]
    fn test_report_savings_by_size() {
        let mut report = SavingsReport::default();

        report.package_savings.push(PackageSavings {
            package_name: "small".to_string(),
            current_size: 10 * 1024,
            potential_savings: 5 * 1024,
            category: SavingsCategory::Underutilized,
            utilization_percentage: Some(10.0),
            exports_used: 1,
            total_exports: Some(10),
            suggestion: "".to_string(),
            alternative: None,
        });

        report.package_savings.push(PackageSavings {
            package_name: "large".to_string(),
            current_size: 100 * 1024,
            potential_savings: 80 * 1024,
            category: SavingsCategory::Unused,
            utilization_percentage: Some(0.0),
            exports_used: 0,
            total_exports: Some(50),
            suggestion: "".to_string(),
            alternative: None,
        });

        let sorted = report.savings_by_size();
        assert_eq!(sorted[0].package_name, "large");
        assert_eq!(sorted[1].package_name, "small");
    }

    #[test]
    fn test_report_savings_by_category() {
        let mut report = SavingsReport::default();

        report.package_savings.push(PackageSavings {
            package_name: "unused1".to_string(),
            current_size: 10 * 1024,
            potential_savings: 10 * 1024,
            category: SavingsCategory::Unused,
            utilization_percentage: None,
            exports_used: 0,
            total_exports: None,
            suggestion: "".to_string(),
            alternative: None,
        });

        report.package_savings.push(PackageSavings {
            package_name: "underutil1".to_string(),
            current_size: 20 * 1024,
            potential_savings: 15 * 1024,
            category: SavingsCategory::Underutilized,
            utilization_percentage: Some(5.0),
            exports_used: 1,
            total_exports: Some(20),
            suggestion: "".to_string(),
            alternative: None,
        });

        let unused = report.savings_by_category(SavingsCategory::Unused);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].package_name, "unused1");

        let underutil = report.savings_by_category(SavingsCategory::Underutilized);
        assert_eq!(underutil.len(), 1);
        assert_eq!(underutil[0].package_name, "underutil1");
    }

    #[test]
    fn test_report_has_savings() {
        let mut report = SavingsReport::default();
        assert!(!report.has_savings());

        report.summary.total_potential_savings = 1024;
        assert!(report.has_savings());
    }

    #[test]
    fn test_report_format() {
        let calc = SavingsCalculator::new();
        let bundle = create_test_bundle_analysis();
        let imports = create_test_project_imports();
        let exports = create_test_export_counts();

        let report = calc.calculate(&bundle, &imports, &exports);
        let formatted = report.format_report();

        assert!(formatted.contains("Bundle Size Savings Report"));
        assert!(formatted.contains("Total Bundle Size:"));
        assert!(formatted.contains("Potential Savings:"));
    }

    #[test]
    fn test_calculate_from_utilization() {
        let calc = SavingsCalculator::new();

        let mut sizes = HashMap::new();
        sizes.insert("pkg-a".to_string(), 100 * 1024); // 100KB
        sizes.insert("pkg-b".to_string(), 50 * 1024); // 50KB
        sizes.insert("moment".to_string(), 300 * 1024); // 300KB - has alternative

        let mut utilization = HashMap::new();
        utilization.insert("pkg-a".to_string(), 5.0); // 5% - underutilized
        utilization.insert("pkg-b".to_string(), 90.0); // 90% - well-utilized
        utilization.insert("moment".to_string(), 50.0);

        let report = calc.calculate_from_utilization(&sizes, &utilization);

        // pkg-a should be underutilized
        let pkg_a = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "pkg-a");
        assert!(pkg_a.is_some());
        assert_eq!(pkg_a.unwrap().category, SavingsCategory::Underutilized);

        // pkg-b should not be in report (well-utilized)
        let pkg_b = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "pkg-b");
        assert!(pkg_b.is_none());

        // moment should suggest alternative
        let moment = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "moment");
        assert!(moment.is_some());
        assert_eq!(moment.unwrap().category, SavingsCategory::HasAlternative);
    }

    #[test]
    fn test_namespace_import_not_reported() {
        let calc = SavingsCalculator::new();

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("namespace-pkg");
        pkg.add_module("namespace-pkg/index.js".to_string(), 50 * 1024);
        analysis.package_sizes.insert("namespace-pkg".to_string(), pkg);

        let mut imports = ProjectImports::new();
        let mut usage = PackageUsage::default();
        usage.uses_namespace = true; // import * as pkg from 'namespace-pkg'
        usage.importing_files.insert("src/app.ts".to_string());
        imports.package_usage.insert("namespace-pkg".to_string(), usage);

        let exports = HashMap::new();

        let report = calc.calculate(&analysis, &imports, &exports);

        // namespace-pkg should NOT be reported (uses all exports)
        let found = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "namespace-pkg");
        assert!(found.is_none());
    }

    #[test]
    fn test_side_effect_import_not_reported() {
        let calc = SavingsCalculator::new();

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("polyfill-pkg");
        pkg.add_module("polyfill-pkg/index.js".to_string(), 20 * 1024);
        analysis.package_sizes.insert("polyfill-pkg".to_string(), pkg);

        let mut imports = ProjectImports::new();
        let mut usage = PackageUsage::default();
        usage.has_side_effects = true; // import 'polyfill-pkg'
        usage.importing_files.insert("src/index.ts".to_string());
        imports.package_usage.insert("polyfill-pkg".to_string(), usage);

        let exports = HashMap::new();

        let report = calc.calculate(&analysis, &imports, &exports);

        // polyfill-pkg should NOT be reported (side-effect import)
        let found = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "polyfill-pkg");
        assert!(found.is_none());
    }

    #[test]
    fn test_small_tree_shaking_not_reported() {
        let calc = SavingsCalculator::new();

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("small-pkg");
        pkg.add_module("small-pkg/index.js".to_string(), 5 * 1024); // 5KB - small
        analysis.package_sizes.insert("small-pkg".to_string(), pkg);

        let mut imports = ProjectImports::new();
        let mut usage = PackageUsage::default();
        usage.named_imports.insert("one".to_string());
        usage.named_imports.insert("two".to_string());
        usage.importing_files.insert("src/app.ts".to_string());
        imports.package_usage.insert("small-pkg".to_string(), usage);

        let mut exports = HashMap::new();
        exports.insert("small-pkg".to_string(), 5); // 2 out of 5 = 40%

        let report = calc.calculate(&analysis, &imports, &exports);

        // small-pkg should NOT be reported (potential savings < 10KB threshold)
        let found = report
            .package_savings
            .iter()
            .find(|s| s.package_name == "small-pkg");
        assert!(found.is_none());
    }
}
