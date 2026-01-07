//! Import analysis using tree-sitter for JavaScript/TypeScript.
//!
//! This module parses source files to extract import statements and track
//! which exports from each dependency are actually used.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use thiserror::Error;
use tree_sitter::{Language, Parser, Tree};
use walkdir::WalkDir;

/// Errors that can occur during import analysis.
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse file: {path}")]
    ParseError { path: String },

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Tree-sitter language initialization failed")]
    LanguageInit,
}

/// Result type for analysis operations.
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// The kind of import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    /// ES6 import statement: `import ... from 'module'`
    ES6,
    /// CommonJS require: `const x = require('module')`
    CommonJS,
    /// Dynamic import: `import('module')`
    DynamicImport,
}

/// An individual import specifier within an import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSpecifier {
    /// Default import: `import foo from 'module'`
    Default(String),
    /// Named import: `import { foo } from 'module'` or `import { foo as bar } from 'module'`
    Named {
        imported: String,
        local: String,
    },
    /// Namespace import: `import * as foo from 'module'`
    Namespace(String),
    /// Side-effect import: `import 'module'` (no specifiers)
    SideEffect,
    /// Entire module (CommonJS style): `const mod = require('module')`
    Entire(String),
}

impl ImportSpecifier {
    /// Returns the exported name that is being imported (the original name in the source module).
    pub fn exported_name(&self) -> Option<&str> {
        match self {
            ImportSpecifier::Default(_) => Some("default"),
            ImportSpecifier::Named { imported, .. } => Some(imported),
            ImportSpecifier::Namespace(_) => None, // Uses all exports
            ImportSpecifier::SideEffect => None,
            ImportSpecifier::Entire(_) => None, // Uses entire module
        }
    }

    /// Returns the local name (the name used in the importing file).
    pub fn local_name(&self) -> Option<&str> {
        match self {
            ImportSpecifier::Default(name) => Some(name),
            ImportSpecifier::Named { local, .. } => Some(local),
            ImportSpecifier::Namespace(name) => Some(name),
            ImportSpecifier::SideEffect => None,
            ImportSpecifier::Entire(name) => Some(name),
        }
    }
}

/// Represents a single import statement in a source file.
#[derive(Debug, Clone)]
pub struct Import {
    /// The source module (e.g., "react", "./utils", "@scope/package")
    pub source: String,
    /// The specifiers being imported
    pub specifiers: Vec<ImportSpecifier>,
    /// The kind of import
    pub kind: ImportKind,
    /// Line number in the source file (1-indexed)
    pub line: usize,
}

impl Import {
    /// Returns true if this import is from an npm package (not a relative/absolute path).
    pub fn is_package_import(&self) -> bool {
        !self.source.starts_with('.') && !self.source.starts_with('/')
    }

    /// Returns the package name for npm imports.
    /// Handles scoped packages like @scope/package.
    pub fn package_name(&self) -> Option<&str> {
        if !self.is_package_import() {
            return None;
        }

        // Handle scoped packages: @scope/package/subpath -> @scope/package
        if self.source.starts_with('@') {
            let parts: Vec<&str> = self.source.splitn(3, '/').collect();
            if parts.len() >= 2 {
                // Return @scope/package
                let end = self.source.find('/').unwrap() + 1 + parts[1].len();
                return Some(&self.source[..end.min(self.source.len())]);
            }
        }

        // Regular package: package/subpath -> package
        if let Some(idx) = self.source.find('/') {
            Some(&self.source[..idx])
        } else {
            Some(&self.source)
        }
    }

    /// Returns true if this is a namespace import (uses all exports).
    pub fn is_namespace_import(&self) -> bool {
        self.specifiers
            .iter()
            .any(|s| matches!(s, ImportSpecifier::Namespace(_)))
    }

    /// Returns true if this is a side-effect only import.
    pub fn is_side_effect_only(&self) -> bool {
        self.specifiers.len() == 1 && matches!(self.specifiers[0], ImportSpecifier::SideEffect)
    }
}

/// Tracks usage information for a single package.
#[derive(Debug, Clone, Default)]
pub struct PackageUsage {
    /// Named exports that are imported from this package.
    pub named_imports: HashSet<String>,
    /// Whether the default export is used.
    pub uses_default: bool,
    /// Whether a namespace import is used (import * as x).
    pub uses_namespace: bool,
    /// Whether there are side-effect imports.
    pub has_side_effects: bool,
    /// Files that import this package.
    pub importing_files: HashSet<String>,
}

impl PackageUsage {
    /// Returns the number of distinct exports being used.
    /// Namespace imports count as "all exports" (represented as -1 in percentage calc).
    pub fn export_count(&self) -> usize {
        let mut count = self.named_imports.len();
        if self.uses_default {
            count += 1;
        }
        count
    }

    /// Calculate utilization percentage given the total number of exports.
    /// Returns 100% if namespace import is used (uses everything).
    /// Returns None if no exports are used (side-effect only).
    pub fn utilization_percentage(&self, total_exports: usize) -> Option<f64> {
        if self.uses_namespace {
            return Some(100.0);
        }
        if total_exports == 0 {
            return None;
        }
        let used = self.export_count();
        Some((used as f64 / total_exports as f64) * 100.0)
    }

    /// Returns true if this package might be underutilized.
    /// A package is potentially underutilized if:
    /// - It doesn't use namespace import
    /// - It uses less than 20% of exports (if we know total)
    pub fn is_potentially_underutilized(&self, total_exports: usize) -> bool {
        if self.uses_namespace || total_exports == 0 {
            return false;
        }
        let used = self.export_count();
        (used as f64 / total_exports as f64) < 0.2
    }
}

/// Collection of all imports found in a project.
#[derive(Debug, Default)]
pub struct ProjectImports {
    /// All imports by file path.
    pub imports_by_file: HashMap<String, Vec<Import>>,
    /// Package usage statistics.
    pub package_usage: HashMap<String, PackageUsage>,
}

impl ProjectImports {
    /// Create a new empty ProjectImports.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add imports from a file.
    pub fn add_file_imports(&mut self, file_path: &str, imports: Vec<Import>) {
        for import in &imports {
            if let Some(pkg_name) = import.package_name() {
                let usage = self.package_usage.entry(pkg_name.to_string()).or_default();
                usage.importing_files.insert(file_path.to_string());

                for spec in &import.specifiers {
                    match spec {
                        ImportSpecifier::Default(_) => {
                            usage.uses_default = true;
                        }
                        ImportSpecifier::Named { imported, .. } => {
                            usage.named_imports.insert(imported.clone());
                        }
                        ImportSpecifier::Namespace(_) => {
                            usage.uses_namespace = true;
                        }
                        ImportSpecifier::SideEffect => {
                            usage.has_side_effects = true;
                        }
                        ImportSpecifier::Entire(_) => {
                            usage.uses_namespace = true; // CommonJS require uses whole module
                        }
                    }
                }
            }
        }

        self.imports_by_file.insert(file_path.to_string(), imports);
    }

    /// Get list of packages sorted by number of importing files (descending).
    pub fn packages_by_usage(&self) -> Vec<(&String, &PackageUsage)> {
        let mut packages: Vec<_> = self.package_usage.iter().collect();
        packages.sort_by(|a, b| b.1.importing_files.len().cmp(&a.1.importing_files.len()));
        packages
    }

    /// Get packages that might be underutilized given export counts.
    pub fn underutilized_packages(
        &self,
        export_counts: &HashMap<String, usize>,
    ) -> Vec<(&String, &PackageUsage, f64)> {
        self.package_usage
            .iter()
            .filter_map(|(name, usage)| {
                let total = export_counts.get(name).copied().unwrap_or(0);
                if total > 0 && usage.is_potentially_underutilized(total) {
                    let percentage = usage.utilization_percentage(total).unwrap_or(0.0);
                    Some((name, usage, percentage))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Language type for file analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    JavaScript,
    TypeScript,
    Tsx,
    Jsx,
}

impl SourceLanguage {
    /// Determine language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" | "cjs" => Some(SourceLanguage::JavaScript),
            "jsx" => Some(SourceLanguage::Jsx),
            "ts" | "mts" | "cts" => Some(SourceLanguage::TypeScript),
            "tsx" => Some(SourceLanguage::Tsx),
            _ => None,
        }
    }

    /// Get tree-sitter language for this source language.
    #[allow(dead_code)]
    pub fn tree_sitter_language(&self) -> Language {
        match self {
            SourceLanguage::JavaScript | SourceLanguage::Jsx => {
                tree_sitter_javascript::LANGUAGE.into()
            }
            SourceLanguage::TypeScript | SourceLanguage::Tsx => {
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
            }
        }
    }
}

/// Analyzer for extracting imports from JavaScript/TypeScript source files.
pub struct ImportAnalyzer {
    js_parser: Parser,
    ts_parser: Parser,
}

impl ImportAnalyzer {
    /// Create a new ImportAnalyzer.
    pub fn new() -> AnalysisResult<Self> {
        let mut js_parser = Parser::new();
        js_parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|_| AnalysisError::LanguageInit)?;

        let mut ts_parser = Parser::new();
        ts_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|_| AnalysisError::LanguageInit)?;

        Ok(Self {
            js_parser,
            ts_parser,
        })
    }

    /// Analyze a single file and extract all imports.
    pub fn analyze_file(&mut self, path: &Path) -> AnalysisResult<Vec<Import>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = SourceLanguage::from_extension(ext)
            .ok_or_else(|| AnalysisError::UnsupportedFileType(ext.to_string()))?;

        let content = fs::read_to_string(path)?;
        self.analyze_source(&content, language, path)
    }

    /// Analyze source code directly.
    pub fn analyze_source(
        &mut self,
        source: &str,
        language: SourceLanguage,
        path: &Path,
    ) -> AnalysisResult<Vec<Import>> {
        let parser = match language {
            SourceLanguage::JavaScript | SourceLanguage::Jsx => &mut self.js_parser,
            SourceLanguage::TypeScript | SourceLanguage::Tsx => &mut self.ts_parser,
        };

        let tree = parser.parse(source, None).ok_or_else(|| AnalysisError::ParseError {
            path: path.display().to_string(),
        })?;

        Ok(self.extract_imports(&tree, source))
    }

    /// Extract imports from a parsed tree.
    fn extract_imports(&self, tree: &Tree, source: &str) -> Vec<Import> {
        let mut imports = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();

        self.visit_node(&mut cursor, source, &mut imports);

        imports
    }

    /// Recursively visit nodes to find imports.
    fn visit_node(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        imports: &mut Vec<Import>,
    ) {
        let node = cursor.node();

        match node.kind() {
            "import_statement" => {
                if let Some(import) = self.parse_es6_import(&node, source) {
                    imports.push(import);
                }
            }
            "call_expression" => {
                // Check for require() or dynamic import()
                if let Some(import) = self.parse_require_or_dynamic_import(&node, source) {
                    imports.push(import);
                }
            }
            _ => {}
        }

        // Visit children
        if cursor.goto_first_child() {
            loop {
                self.visit_node(cursor, source, imports);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    /// Parse an ES6 import statement.
    fn parse_es6_import(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let mut source_module = String::new();
        let mut specifiers = Vec::new();
        let line = node.start_position().row + 1;

        let mut cursor = node.walk();

        // Find the source (string after 'from')
        for child in node.children(&mut cursor) {
            match child.kind() {
                "string" => {
                    source_module = self.extract_string_value(&child, source)?;
                }
                "import_clause" => {
                    self.parse_import_clause(&child, source, &mut specifiers);
                }
                _ => {}
            }
        }

        // Side-effect import if no specifiers
        if specifiers.is_empty() && !source_module.is_empty() {
            specifiers.push(ImportSpecifier::SideEffect);
        }

        if source_module.is_empty() {
            return None;
        }

        Some(Import {
            source: source_module,
            specifiers,
            kind: ImportKind::ES6,
            line,
        })
    }

    /// Parse the import clause (everything between 'import' and 'from').
    fn parse_import_clause(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        specifiers: &mut Vec<ImportSpecifier>,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Default import: import foo from 'module'
                    if let Some(name) = self.node_text(&child, source) {
                        specifiers.push(ImportSpecifier::Default(name.to_string()));
                    }
                }
                "namespace_import" => {
                    // Namespace import: import * as foo from 'module'
                    if let Some(name) = self.find_namespace_name(&child, source) {
                        specifiers.push(ImportSpecifier::Namespace(name));
                    }
                }
                "named_imports" => {
                    // Named imports: import { foo, bar as baz } from 'module'
                    self.parse_named_imports(&child, source, specifiers);
                }
                _ => {}
            }
        }
    }

    /// Find the local name in a namespace import (import * as NAME).
    fn find_namespace_name(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return self.node_text(&child, source).map(|s| s.to_string());
            }
        }
        None
    }

    /// Parse named imports: { foo, bar as baz, default as qux }
    fn parse_named_imports(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        specifiers: &mut Vec<ImportSpecifier>,
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "import_specifier" {
                if let Some(spec) = self.parse_import_specifier(&child, source) {
                    specifiers.push(spec);
                }
            }
        }
    }

    /// Parse a single import specifier: foo or foo as bar
    fn parse_import_specifier(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<ImportSpecifier> {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        // Check for "name as alias" pattern
        let mut imported = None;
        let mut local = None;

        for child in children.iter() {
            if child.kind() == "identifier" {
                let name = self.node_text(child, source)?;
                if imported.is_none() {
                    imported = Some(name.to_string());
                } else {
                    local = Some(name.to_string());
                }
            }
        }

        let imported = imported?;
        let local = local.unwrap_or_else(|| imported.clone());

        Some(ImportSpecifier::Named { imported, local })
    }

    /// Parse require() calls or dynamic import().
    fn parse_require_or_dynamic_import(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<Import> {
        let line = node.start_position().row + 1;

        // Get the function being called
        let func_node = node.child_by_field_name("function")?;
        let func_name = self.node_text(&func_node, source)?;

        let (kind, is_require) = match func_name {
            "require" => (ImportKind::CommonJS, true),
            "import" => (ImportKind::DynamicImport, false),
            _ => return None,
        };

        // Get arguments
        let args_node = node.child_by_field_name("arguments")?;
        let mut args_cursor = args_node.walk();

        for child in args_node.children(&mut args_cursor) {
            if child.kind() == "string" {
                let source_module = self.extract_string_value(&child, source)?;

                // For CommonJS require, try to find the variable name
                let specifiers = if is_require {
                    self.find_require_variable_name(node, source)
                        .map(|name| vec![ImportSpecifier::Entire(name)])
                        .unwrap_or_else(|| vec![ImportSpecifier::SideEffect])
                } else {
                    vec![ImportSpecifier::SideEffect]
                };

                return Some(Import {
                    source: source_module,
                    specifiers,
                    kind,
                    line,
                });
            }
        }

        None
    }

    /// Find the variable name in `const x = require('...')`.
    fn find_require_variable_name(
        &self,
        call_node: &tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        // Go up to find variable_declarator or lexical_declaration
        let parent = call_node.parent()?;

        match parent.kind() {
            "variable_declarator" => {
                // const x = require('...')
                let name_node = parent.child_by_field_name("name")?;
                match name_node.kind() {
                    "identifier" => self.node_text(&name_node, source).map(|s| s.to_string()),
                    "object_pattern" | "array_pattern" => {
                        // Destructuring: const { x, y } = require('...')
                        // For now, treat as namespace import
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Extract the text content of a node.
    fn node_text<'a>(&self, node: &tree_sitter::Node, source: &'a str) -> Option<&'a str> {
        let start = node.start_byte();
        let end = node.end_byte();
        source.get(start..end)
    }

    /// Extract string value (removes quotes).
    fn extract_string_value(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let text = self.node_text(node, source)?;
        // Remove quotes (single, double, or backticks)
        let trimmed = text
            .trim_start_matches(['"', '\'', '`'])
            .trim_end_matches(['"', '\'', '`']);
        Some(trimmed.to_string())
    }
}

impl Default for ImportAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ImportAnalyzer")
    }
}

/// Analyze a single file and return its imports.
pub fn analyze_file(path: &Path) -> AnalysisResult<Vec<Import>> {
    let mut analyzer = ImportAnalyzer::new()?;
    analyzer.analyze_file(path)
}

/// Analyze all JavaScript/TypeScript files in a directory.
pub fn analyze_project_imports(root: &Path) -> AnalysisResult<ProjectImports> {
    let mut analyzer = ImportAnalyzer::new()?;
    let mut project = ProjectImports::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check if it's a supported file type
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if SourceLanguage::from_extension(ext).is_none() {
            continue;
        }

        match analyzer.analyze_file(path) {
            Ok(imports) => {
                let file_path = path.display().to_string();
                project.add_file_imports(&file_path, imports);
            }
            Err(e) => {
                // Log error but continue with other files
                eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
            }
        }
    }

    Ok(project)
}

/// Check if a directory should be ignored during traversal.
fn is_ignored_dir(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }

    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        "node_modules" | ".git" | "dist" | "build" | ".next" | "coverage" | ".turbo"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_source(source: &str) -> Vec<Import> {
        let mut analyzer = ImportAnalyzer::new().unwrap();
        analyzer
            .analyze_source(source, SourceLanguage::JavaScript, Path::new("test.js"))
            .unwrap()
    }

    fn parse_ts_source(source: &str) -> Vec<Import> {
        let mut analyzer = ImportAnalyzer::new().unwrap();
        analyzer
            .analyze_source(source, SourceLanguage::TypeScript, Path::new("test.ts"))
            .unwrap()
    }

    // ===== ES6 Import Tests =====

    #[test]
    fn test_default_import() {
        let source = r#"import React from 'react';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
        assert_eq!(imports[0].kind, ImportKind::ES6);
        assert_eq!(imports[0].specifiers.len(), 1);
        assert!(matches!(
            &imports[0].specifiers[0],
            ImportSpecifier::Default(name) if name == "React"
        ));
    }

    #[test]
    fn test_named_imports() {
        let source = r#"import { useState, useEffect } from 'react';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
        assert_eq!(imports[0].specifiers.len(), 2);

        let names: Vec<_> = imports[0]
            .specifiers
            .iter()
            .filter_map(|s| s.exported_name())
            .collect();
        assert!(names.contains(&"useState"));
        assert!(names.contains(&"useEffect"));
    }

    #[test]
    fn test_named_import_with_alias() {
        let source = r#"import { useState as state } from 'react';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].specifiers.len(), 1);
        assert!(matches!(
            &imports[0].specifiers[0],
            ImportSpecifier::Named { imported, local }
                if imported == "useState" && local == "state"
        ));
    }

    #[test]
    fn test_namespace_import() {
        let source = r#"import * as React from 'react';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
        assert!(matches!(
            &imports[0].specifiers[0],
            ImportSpecifier::Namespace(name) if name == "React"
        ));
        assert!(imports[0].is_namespace_import());
    }

    #[test]
    fn test_mixed_imports() {
        let source = r#"import React, { useState, useEffect } from 'react';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].specifiers.len(), 3);

        let has_default = imports[0]
            .specifiers
            .iter()
            .any(|s| matches!(s, ImportSpecifier::Default(_)));
        assert!(has_default);

        let named_count = imports[0]
            .specifiers
            .iter()
            .filter(|s| matches!(s, ImportSpecifier::Named { .. }))
            .count();
        assert_eq!(named_count, 2);
    }

    #[test]
    fn test_side_effect_import() {
        let source = r#"import './styles.css';"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "./styles.css");
        assert!(imports[0].is_side_effect_only());
    }

    // ===== CommonJS Tests =====

    #[test]
    fn test_require_simple() {
        let source = r#"const React = require('react');"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
        assert_eq!(imports[0].kind, ImportKind::CommonJS);
        assert!(matches!(
            &imports[0].specifiers[0],
            ImportSpecifier::Entire(name) if name == "React"
        ));
    }

    #[test]
    fn test_require_without_assignment() {
        let source = r#"require('./polyfills');"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "./polyfills");
        assert_eq!(imports[0].kind, ImportKind::CommonJS);
        assert!(imports[0].is_side_effect_only());
    }

    // ===== Package Name Tests =====

    #[test]
    fn test_package_name_simple() {
        let import = Import {
            source: "react".to_string(),
            specifiers: vec![ImportSpecifier::Default("React".to_string())],
            kind: ImportKind::ES6,
            line: 1,
        };
        assert_eq!(import.package_name(), Some("react"));
    }

    #[test]
    fn test_package_name_with_subpath() {
        let import = Import {
            source: "lodash/debounce".to_string(),
            specifiers: vec![ImportSpecifier::Default("debounce".to_string())],
            kind: ImportKind::ES6,
            line: 1,
        };
        assert_eq!(import.package_name(), Some("lodash"));
    }

    #[test]
    fn test_package_name_scoped() {
        let import = Import {
            source: "@tanstack/react-query".to_string(),
            specifiers: vec![],
            kind: ImportKind::ES6,
            line: 1,
        };
        assert_eq!(import.package_name(), Some("@tanstack/react-query"));
    }

    #[test]
    fn test_package_name_scoped_with_subpath() {
        let import = Import {
            source: "@tanstack/react-query/devtools".to_string(),
            specifiers: vec![],
            kind: ImportKind::ES6,
            line: 1,
        };
        assert_eq!(import.package_name(), Some("@tanstack/react-query"));
    }

    #[test]
    fn test_relative_import_no_package() {
        let import = Import {
            source: "./utils".to_string(),
            specifiers: vec![],
            kind: ImportKind::ES6,
            line: 1,
        };
        assert_eq!(import.package_name(), None);
    }

    // ===== TypeScript Tests =====

    #[test]
    fn test_typescript_type_import() {
        let source = r#"import type { FC } from 'react';"#;
        let imports = parse_ts_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "react");
    }

    // ===== ProjectImports Tests =====

    #[test]
    fn test_project_imports_aggregation() {
        let mut project = ProjectImports::new();

        let imports1 = vec![Import {
            source: "react".to_string(),
            specifiers: vec![
                ImportSpecifier::Named {
                    imported: "useState".to_string(),
                    local: "useState".to_string(),
                },
            ],
            kind: ImportKind::ES6,
            line: 1,
        }];

        let imports2 = vec![Import {
            source: "react".to_string(),
            specifiers: vec![
                ImportSpecifier::Named {
                    imported: "useEffect".to_string(),
                    local: "useEffect".to_string(),
                },
            ],
            kind: ImportKind::ES6,
            line: 1,
        }];

        project.add_file_imports("file1.js", imports1);
        project.add_file_imports("file2.js", imports2);

        let react_usage = project.package_usage.get("react").unwrap();
        assert_eq!(react_usage.named_imports.len(), 2);
        assert!(react_usage.named_imports.contains("useState"));
        assert!(react_usage.named_imports.contains("useEffect"));
        assert_eq!(react_usage.importing_files.len(), 2);
    }

    #[test]
    fn test_utilization_percentage() {
        let mut usage = PackageUsage::default();
        usage.named_imports.insert("foo".to_string());
        usage.named_imports.insert("bar".to_string());

        // 2 out of 10 exports = 20%
        assert_eq!(usage.utilization_percentage(10), Some(20.0));

        // With default export: 3 out of 10 = 30%
        usage.uses_default = true;
        assert_eq!(usage.utilization_percentage(10), Some(30.0));

        // Namespace import = 100%
        usage.uses_namespace = true;
        assert_eq!(usage.utilization_percentage(10), Some(100.0));
    }

    #[test]
    fn test_underutilized_detection() {
        let mut usage = PackageUsage::default();
        usage.named_imports.insert("foo".to_string());

        // 1 out of 10 = 10% < 20% threshold
        assert!(usage.is_potentially_underutilized(10));

        // 1 out of 5 = 20% = threshold (not underutilized)
        assert!(!usage.is_potentially_underutilized(5));

        // Namespace import is never underutilized
        usage.uses_namespace = true;
        assert!(!usage.is_potentially_underutilized(10));
    }

    // ===== Multiple Import Statements =====

    #[test]
    fn test_multiple_imports() {
        let source = r#"
import React from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import './styles.css';
"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 4);

        let packages: Vec<_> = imports.iter().map(|i| i.source.as_str()).collect();
        assert!(packages.contains(&"react"));
        assert!(packages.contains(&"@tanstack/react-query"));
        assert!(packages.contains(&"axios"));
        assert!(packages.contains(&"./styles.css"));
    }

    // ===== Dynamic Import Tests =====

    #[test]
    fn test_dynamic_import() {
        let source = r#"const module = await import('lodash');"#;
        let imports = parse_source(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source, "lodash");
        assert_eq!(imports[0].kind, ImportKind::DynamicImport);
    }
}
