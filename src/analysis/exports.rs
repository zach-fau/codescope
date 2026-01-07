//! Export and import analysis using tree-sitter.
//!
//! This module provides functionality to parse JavaScript and TypeScript files
//! to extract import statements and track which symbols are imported from each package.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during import analysis.
#[derive(Debug, Error)]
pub enum AnalysisError {
    /// Failed to read the source file.
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse the source file.
    #[error("Failed to parse file: {0}")]
    ParseError(String),

    /// Unsupported file type.
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Result type for analysis operations.
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Represents the style of import used.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportStyle {
    /// Named imports: `import { foo, bar } from 'package'`
    Named,
    /// Default import: `import foo from 'package'`
    Default,
    /// Namespace import: `import * as foo from 'package'`
    Namespace,
    /// Side-effect import: `import 'package'`
    SideEffect,
    /// CommonJS require: `const foo = require('package')`
    CommonJs,
    /// Dynamic import: `import('package')`
    Dynamic,
}

impl std::fmt::Display for ImportStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ImportStyle::Named => "named",
            ImportStyle::Default => "default",
            ImportStyle::Namespace => "namespace",
            ImportStyle::SideEffect => "side-effect",
            ImportStyle::CommonJs => "commonjs",
            ImportStyle::Dynamic => "dynamic",
        };
        write!(f, "{}", s)
    }
}

/// Information about a single import statement.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// The package name being imported from (e.g., "react", "lodash").
    pub package_name: String,

    /// The symbols imported from this package.
    /// Empty for namespace, side-effect, or default imports.
    pub imported_symbols: Vec<String>,

    /// The style of import used.
    pub import_style: ImportStyle,

    /// The local alias if different from the original name.
    /// For `import { foo as bar }`, this would be Some("bar").
    pub alias: Option<String>,

    /// The line number where this import appears.
    pub line: usize,
}

impl ImportInfo {
    /// Creates a new ImportInfo instance.
    pub fn new(package_name: String, import_style: ImportStyle) -> Self {
        Self {
            package_name,
            imported_symbols: Vec::new(),
            import_style,
            alias: None,
            line: 0,
        }
    }

    /// Returns true if this is a local/relative import (starts with . or /).
    pub fn is_local(&self) -> bool {
        self.package_name.starts_with('.')
            || self.package_name.starts_with('/')
            || self.package_name.starts_with("@/")
    }

    /// Returns true if this is an external package import.
    pub fn is_external(&self) -> bool {
        !self.is_local()
    }
}

/// Aggregated import usage for a single package across the project.
#[derive(Debug, Clone)]
pub struct ImportUsage {
    /// The package name.
    pub package_name: String,

    /// All unique symbols imported from this package.
    pub imported_symbols: HashSet<String>,

    /// Number of files that import this package.
    pub import_count: usize,

    /// The import styles used for this package.
    pub import_styles: HashSet<ImportStyle>,

    /// Whether the package is ever imported with namespace import.
    pub has_namespace_import: bool,

    /// Whether the package is ever imported with side-effect import.
    pub has_side_effect_import: bool,
}

impl ImportUsage {
    /// Creates a new ImportUsage instance.
    pub fn new(package_name: String) -> Self {
        Self {
            package_name,
            imported_symbols: HashSet::new(),
            import_count: 0,
            import_styles: HashSet::new(),
            has_namespace_import: false,
            has_side_effect_import: false,
        }
    }

    /// Merges another import into this usage tracker.
    pub fn merge(&mut self, import: &ImportInfo) {
        self.import_count += 1;
        self.import_styles.insert(import.import_style.clone());

        for symbol in &import.imported_symbols {
            self.imported_symbols.insert(symbol.clone());
        }

        if import.import_style == ImportStyle::Namespace {
            self.has_namespace_import = true;
        }

        if import.import_style == ImportStyle::SideEffect {
            self.has_side_effect_import = true;
        }
    }

    /// Returns true if we can determine utilization (not namespace/side-effect).
    pub fn can_calculate_utilization(&self) -> bool {
        !self.has_namespace_import && !self.has_side_effect_import
    }

    /// Returns the number of unique symbols imported.
    pub fn symbol_count(&self) -> usize {
        self.imported_symbols.len()
    }
}

/// Aggregated imports for an entire project.
#[derive(Debug, Clone, Default)]
pub struct ProjectImports {
    /// Map of package name to its import usage.
    pub packages: HashMap<String, ImportUsage>,

    /// Total number of files analyzed.
    pub files_analyzed: usize,

    /// Files that failed to parse.
    pub parse_errors: Vec<String>,
}

impl ProjectImports {
    /// Creates a new ProjectImports instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds imports from a single file to the project imports.
    pub fn add_file_imports(&mut self, imports: Vec<ImportInfo>) {
        self.files_analyzed += 1;

        for import in imports {
            // Skip local imports
            if import.is_local() {
                continue;
            }

            let usage = self
                .packages
                .entry(import.package_name.clone())
                .or_insert_with(|| ImportUsage::new(import.package_name.clone()));

            usage.merge(&import);
        }
    }

    /// Records a parse error for a file.
    pub fn add_parse_error(&mut self, file_path: String) {
        self.parse_errors.push(file_path);
    }

    /// Returns packages that have zero named imports (potential unused dependencies).
    pub fn packages_with_zero_imports(&self) -> Vec<&ImportUsage> {
        self.packages
            .values()
            .filter(|usage| {
                usage.imported_symbols.is_empty()
                    && !usage.has_namespace_import
                    && !usage.has_side_effect_import
            })
            .collect()
    }

    /// Returns all external package names that are imported.
    pub fn imported_packages(&self) -> Vec<&str> {
        self.packages.keys().map(|s| s.as_str()).collect()
    }

    /// Returns the usage info for a specific package.
    pub fn get_package(&self, name: &str) -> Option<&ImportUsage> {
        self.packages.get(name)
    }
}

/// Analyzer for extracting import information from JavaScript/TypeScript files.
pub struct ImportAnalyzer {
    js_parser: tree_sitter::Parser,
    ts_parser: tree_sitter::Parser,
    tsx_parser: tree_sitter::Parser,
}

impl ImportAnalyzer {
    /// Creates a new ImportAnalyzer with JavaScript and TypeScript parsers.
    pub fn new() -> Self {
        let mut js_parser = tree_sitter::Parser::new();
        js_parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load JavaScript grammar");

        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Failed to load TypeScript grammar");

        let mut tsx_parser = tree_sitter::Parser::new();
        tsx_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
            .expect("Failed to load TSX grammar");

        Self {
            js_parser,
            ts_parser,
            tsx_parser,
        }
    }

    /// Determines the file type based on extension.
    fn get_file_type(path: &Path) -> Option<FileType> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "js" | "mjs" | "cjs" | "jsx" => Some(FileType::JavaScript),
            "ts" | "mts" | "cts" => Some(FileType::TypeScript),
            "tsx" => Some(FileType::Tsx),
            _ => None,
        }
    }

    /// Analyzes a single file and returns all import information.
    pub fn analyze_file(&mut self, path: &Path) -> AnalysisResult<Vec<ImportInfo>> {
        let file_type = Self::get_file_type(path)
            .ok_or_else(|| AnalysisError::UnsupportedFileType(path.display().to_string()))?;

        let source = fs::read_to_string(path)?;
        self.analyze_source(&source, file_type)
    }

    /// Analyzes source code and returns all import information.
    pub fn analyze_source(
        &mut self,
        source: &str,
        file_type: FileType,
    ) -> AnalysisResult<Vec<ImportInfo>> {
        let parser = match file_type {
            FileType::JavaScript => &mut self.js_parser,
            FileType::TypeScript => &mut self.ts_parser,
            FileType::Tsx => &mut self.tsx_parser,
        };

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| AnalysisError::ParseError("Failed to parse source".to_string()))?;

        let mut imports = Vec::new();
        let mut cursor = tree.walk();

        self.extract_imports(&mut cursor, source.as_bytes(), &mut imports);

        Ok(imports)
    }

    /// Recursively extracts imports from the AST.
    fn extract_imports(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &[u8],
        imports: &mut Vec<ImportInfo>,
    ) {
        loop {
            let node = cursor.node();

            match node.kind() {
                "import_statement" => {
                    if let Some(import) = self.parse_import_statement(node, source) {
                        imports.push(import);
                    }
                }
                "call_expression" => {
                    // Check for dynamic import() only (not require - that's handled in variable_declaration)
                    // Also handle standalone require() calls that aren't assigned to variables
                    if let Some(import) = self.parse_call_expression(node, source) {
                        // Only add if it's a dynamic import OR if it's a standalone require (not inside a variable declaration)
                        let parent_kind = node.parent().map(|p| p.kind());
                        let is_in_variable_declarator = parent_kind == Some("variable_declarator");

                        if import.import_style == ImportStyle::Dynamic || !is_in_variable_declarator {
                            // For require that IS in a variable declarator, skip it (handled by variable_declaration case)
                            if import.import_style == ImportStyle::CommonJs && is_in_variable_declarator {
                                // Skip - will be handled by parse_variable_require
                            } else {
                                imports.push(import);
                            }
                        }
                    }
                }
                "lexical_declaration" | "variable_declaration" => {
                    // Check for `const x = require('...')`
                    if let Some(import) = self.parse_variable_require(node, source) {
                        imports.push(import);
                    }
                }
                _ => {}
            }

            // Recurse into children
            if cursor.goto_first_child() {
                self.extract_imports(cursor, source, imports);
                cursor.goto_parent();
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    /// Parses an ES6 import statement.
    fn parse_import_statement(
        &self,
        node: tree_sitter::Node,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut import_clause = None;
        let mut source_node = None;

        let mut cursor = node.walk();
        cursor.goto_first_child();

        loop {
            let child = cursor.node();
            match child.kind() {
                "import_clause" => import_clause = Some(child),
                "string" | "string_fragment" => source_node = Some(child),
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        // Get the module source (package name)
        let source_node = source_node?;
        let package_name = self.get_string_content(source_node, source)?;

        let line = node.start_position().row + 1;

        // Side-effect import: `import 'package'`
        if import_clause.is_none() {
            return Some(ImportInfo {
                package_name,
                imported_symbols: Vec::new(),
                import_style: ImportStyle::SideEffect,
                alias: None,
                line,
            });
        }

        let import_clause = import_clause?;
        let (symbols, style, alias) = self.parse_import_clause(import_clause, source);

        Some(ImportInfo {
            package_name,
            imported_symbols: symbols,
            import_style: style,
            alias,
            line,
        })
    }

    /// Parses the import clause to extract symbols and style.
    fn parse_import_clause(
        &self,
        node: tree_sitter::Node,
        source: &[u8],
    ) -> (Vec<String>, ImportStyle, Option<String>) {
        let mut symbols = Vec::new();
        let mut style = ImportStyle::Default;
        let mut alias = None;

        let mut cursor = node.walk();
        cursor.goto_first_child();

        loop {
            let child = cursor.node();
            match child.kind() {
                "identifier" => {
                    // Default import: `import foo from 'package'`
                    if let Ok(name) = child.utf8_text(source) {
                        alias = Some(name.to_string());
                        style = ImportStyle::Default;
                    }
                }
                "namespace_import" => {
                    // Namespace import: `import * as foo from 'package'`
                    style = ImportStyle::Namespace;
                    if let Some(name) = self.get_namespace_alias(child, source) {
                        alias = Some(name);
                    }
                }
                "named_imports" => {
                    // Named imports: `import { foo, bar } from 'package'`
                    style = ImportStyle::Named;
                    symbols = self.parse_named_imports(child, source);
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        (symbols, style, alias)
    }

    /// Parses named imports to extract symbol names.
    fn parse_named_imports(&self, node: tree_sitter::Node, source: &[u8]) -> Vec<String> {
        let mut symbols = Vec::new();
        let mut cursor = node.walk();

        if !cursor.goto_first_child() {
            return symbols;
        }

        loop {
            let child = cursor.node();
            if child.kind() == "import_specifier" {
                if let Some(name) = self.get_import_specifier_name(child, source) {
                    symbols.push(name);
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        symbols
    }

    /// Gets the original name from an import specifier.
    fn get_import_specifier_name(&self, node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        // The first identifier is the original name
        // In `import { foo as bar }`, we want "foo"
        loop {
            let child = cursor.node();
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        None
    }

    /// Gets the alias from a namespace import.
    fn get_namespace_alias(&self, node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        loop {
            let child = cursor.node();
            if child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        None
    }

    /// Parses a call expression for require() or dynamic import().
    fn parse_call_expression(
        &self,
        node: tree_sitter::Node,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut cursor = node.walk();
        cursor.goto_first_child();

        let function_node = cursor.node();
        let function_name = function_node.utf8_text(source).ok()?;

        // Check for dynamic import()
        if function_name == "import" {
            cursor.goto_next_sibling(); // Move to arguments
            let args = cursor.node();
            if args.kind() == "arguments" {
                let package_name = self.get_first_string_arg(args, source)?;
                return Some(ImportInfo {
                    package_name,
                    imported_symbols: Vec::new(),
                    import_style: ImportStyle::Dynamic,
                    alias: None,
                    line: node.start_position().row + 1,
                });
            }
        }

        // Check for require()
        if function_name == "require" {
            cursor.goto_next_sibling(); // Move to arguments
            let args = cursor.node();
            if args.kind() == "arguments" {
                let package_name = self.get_first_string_arg(args, source)?;
                return Some(ImportInfo {
                    package_name,
                    imported_symbols: Vec::new(),
                    import_style: ImportStyle::CommonJs,
                    alias: None,
                    line: node.start_position().row + 1,
                });
            }
        }

        None
    }

    /// Parses variable declarations that use require().
    fn parse_variable_require(
        &self,
        node: tree_sitter::Node,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        loop {
            let child = cursor.node();
            if child.kind() == "variable_declarator" {
                return self.parse_variable_declarator_require(child, source);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        None
    }

    /// Parses a variable declarator for require() patterns.
    fn parse_variable_declarator_require(
        &self,
        node: tree_sitter::Node,
        source: &[u8],
    ) -> Option<ImportInfo> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        let mut pattern_node = None;
        let mut value_node = None;

        loop {
            let child = cursor.node();
            match child.kind() {
                "identifier" | "object_pattern" | "array_pattern" => {
                    if pattern_node.is_none() {
                        pattern_node = Some(child);
                    }
                }
                "call_expression" => {
                    value_node = Some(child);
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        let value_node = value_node?;

        // Check if the call expression is require()
        let mut cursor = value_node.walk();
        cursor.goto_first_child();
        let function_node = cursor.node();

        if function_node.utf8_text(source).ok()? != "require" {
            return None;
        }

        cursor.goto_next_sibling();
        let args = cursor.node();
        if args.kind() != "arguments" {
            return None;
        }

        let package_name = self.get_first_string_arg(args, source)?;
        let line = node.start_position().row + 1;

        // Check if destructured: `const { foo, bar } = require('package')`
        let (symbols, alias) = match pattern_node {
            Some(pattern) if pattern.kind() == "object_pattern" => {
                let symbols = self.parse_object_pattern(pattern, source);
                (symbols, None)
            }
            Some(pattern) if pattern.kind() == "identifier" => {
                let alias = pattern.utf8_text(source).ok().map(|s| s.to_string());
                (Vec::new(), alias)
            }
            _ => (Vec::new(), None),
        };

        let style = if symbols.is_empty() {
            ImportStyle::CommonJs
        } else {
            ImportStyle::Named
        };

        Some(ImportInfo {
            package_name,
            imported_symbols: symbols,
            import_style: style,
            alias,
            line,
        })
    }

    /// Parses an object pattern to extract destructured names.
    fn parse_object_pattern(&self, node: tree_sitter::Node, source: &[u8]) -> Vec<String> {
        let mut names = Vec::new();
        let mut cursor = node.walk();

        if !cursor.goto_first_child() {
            return names;
        }

        loop {
            let child = cursor.node();
            match child.kind() {
                "shorthand_property_identifier_pattern" | "shorthand_property_identifier" => {
                    if let Ok(name) = child.utf8_text(source) {
                        names.push(name.to_string());
                    }
                }
                "pair_pattern" => {
                    // Handle `{ foo: bar }` pattern - we want "foo"
                    if let Some(name) = self.get_pair_pattern_key(child, source) {
                        names.push(name);
                    }
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        names
    }

    /// Gets the key from a pair pattern.
    fn get_pair_pattern_key(&self, node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        loop {
            let child = cursor.node();
            if child.kind() == "property_identifier" || child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        None
    }

    /// Gets the first string argument from an arguments node.
    fn get_first_string_arg(&self, node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        if !cursor.goto_first_child() {
            return None;
        }

        loop {
            let child = cursor.node();
            if child.kind() == "string" {
                return self.get_string_content(child, source);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        None
    }

    /// Extracts the content from a string node (removing quotes).
    fn get_string_content(&self, node: tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();

        // Try to find string_fragment child
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "string_fragment" {
                    return child.utf8_text(source).ok().map(|s| s.to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Fallback: extract from the string node directly (remove quotes)
        let text = node.utf8_text(source).ok()?;
        if (text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('`') && text.ends_with('`'))
        {
            Some(text[1..text.len() - 1].to_string())
        } else {
            Some(text.to_string())
        }
    }
}

impl Default for ImportAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// The type of JavaScript/TypeScript file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// JavaScript (.js, .mjs, .cjs, .jsx)
    JavaScript,
    /// TypeScript (.ts, .mts, .cts)
    TypeScript,
    /// TSX (.tsx)
    Tsx,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_js(source: &str) -> Vec<ImportInfo> {
        let mut analyzer = ImportAnalyzer::new();
        analyzer
            .analyze_source(source, FileType::JavaScript)
            .unwrap()
    }

    fn analyze_ts(source: &str) -> Vec<ImportInfo> {
        let mut analyzer = ImportAnalyzer::new();
        analyzer
            .analyze_source(source, FileType::TypeScript)
            .unwrap()
    }

    #[test]
    fn test_named_imports() {
        let source = r#"import { useState, useEffect } from 'react';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "react");
        assert_eq!(imports[0].import_style, ImportStyle::Named);
        assert_eq!(imports[0].imported_symbols, vec!["useState", "useEffect"]);
    }

    #[test]
    fn test_default_import() {
        let source = r#"import lodash from 'lodash';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "lodash");
        assert_eq!(imports[0].import_style, ImportStyle::Default);
        assert_eq!(imports[0].alias, Some("lodash".to_string()));
    }

    #[test]
    fn test_namespace_import() {
        let source = r#"import * as utils from './utils';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "./utils");
        assert_eq!(imports[0].import_style, ImportStyle::Namespace);
        assert_eq!(imports[0].alias, Some("utils".to_string()));
    }

    #[test]
    fn test_side_effect_import() {
        let source = r#"import 'polyfill';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "polyfill");
        assert_eq!(imports[0].import_style, ImportStyle::SideEffect);
    }

    #[test]
    fn test_commonjs_require() {
        let source = r#"const fs = require('fs');"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "fs");
        assert_eq!(imports[0].import_style, ImportStyle::CommonJs);
        assert_eq!(imports[0].alias, Some("fs".to_string()));
    }

    #[test]
    fn test_destructured_require() {
        let source = r#"const { readFile, writeFile } = require('fs');"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "fs");
        assert_eq!(imports[0].import_style, ImportStyle::Named);
        assert!(imports[0].imported_symbols.contains(&"readFile".to_string()));
        assert!(imports[0].imported_symbols.contains(&"writeFile".to_string()));
    }

    #[test]
    fn test_mixed_imports() {
        let source = r#"
            import React, { useState } from 'react';
            import * as lodash from 'lodash';
            import 'polyfill';
            const fs = require('fs');
        "#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 4);

        // React import (default + named combined)
        let react_import = imports.iter().find(|i| i.package_name == "react").unwrap();
        assert!(
            react_import.import_style == ImportStyle::Default
                || react_import.import_style == ImportStyle::Named
        );

        // Lodash namespace
        let lodash_import = imports.iter().find(|i| i.package_name == "lodash").unwrap();
        assert_eq!(lodash_import.import_style, ImportStyle::Namespace);

        // Polyfill side-effect
        let polyfill_import = imports.iter().find(|i| i.package_name == "polyfill").unwrap();
        assert_eq!(polyfill_import.import_style, ImportStyle::SideEffect);

        // fs require
        let fs_import = imports.iter().find(|i| i.package_name == "fs").unwrap();
        assert_eq!(fs_import.import_style, ImportStyle::CommonJs);
    }

    #[test]
    fn test_typescript_imports() {
        let source = r#"
            import type { FC } from 'react';
            import { useState } from 'react';
        "#;
        let imports = analyze_ts(source);

        // Should capture both imports
        assert!(imports.len() >= 1);
        let react_imports: Vec<_> = imports.iter().filter(|i| i.package_name == "react").collect();
        assert!(!react_imports.is_empty());
    }

    #[test]
    fn test_scoped_package() {
        let source = r#"import { Button } from '@mui/material';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "@mui/material");
        assert_eq!(imports[0].imported_symbols, vec!["Button"]);
    }

    #[test]
    fn test_local_import_detection() {
        let imports = analyze_js(r#"import { foo } from './local';"#);
        assert!(imports[0].is_local());
        assert!(!imports[0].is_external());

        let imports = analyze_js(r#"import { bar } from 'external-pkg';"#);
        assert!(!imports[0].is_local());
        assert!(imports[0].is_external());
    }

    #[test]
    fn test_project_imports_aggregation() {
        let mut project = ProjectImports::new();

        // File 1
        project.add_file_imports(vec![
            ImportInfo {
                package_name: "react".to_string(),
                imported_symbols: vec!["useState".to_string()],
                import_style: ImportStyle::Named,
                alias: None,
                line: 1,
            },
            ImportInfo {
                package_name: "./local".to_string(),
                imported_symbols: vec!["foo".to_string()],
                import_style: ImportStyle::Named,
                alias: None,
                line: 2,
            },
        ]);

        // File 2
        project.add_file_imports(vec![ImportInfo {
            package_name: "react".to_string(),
            imported_symbols: vec!["useEffect".to_string()],
            import_style: ImportStyle::Named,
            alias: None,
            line: 1,
        }]);

        assert_eq!(project.files_analyzed, 2);
        assert_eq!(project.packages.len(), 1); // Only external packages

        let react_usage = project.get_package("react").unwrap();
        assert_eq!(react_usage.import_count, 2);
        assert!(react_usage.imported_symbols.contains("useState"));
        assert!(react_usage.imported_symbols.contains("useEffect"));
    }

    #[test]
    fn test_import_usage_utilization() {
        let mut usage = ImportUsage::new("test".to_string());

        // Named imports can calculate utilization
        usage.merge(&ImportInfo {
            package_name: "test".to_string(),
            imported_symbols: vec!["foo".to_string()],
            import_style: ImportStyle::Named,
            alias: None,
            line: 1,
        });
        assert!(usage.can_calculate_utilization());

        // Namespace imports cannot
        usage.merge(&ImportInfo {
            package_name: "test".to_string(),
            imported_symbols: vec![],
            import_style: ImportStyle::Namespace,
            alias: Some("test".to_string()),
            line: 2,
        });
        assert!(!usage.can_calculate_utilization());
    }

    #[test]
    fn test_aliased_import() {
        let source = r#"import { useState as state } from 'react';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        // We capture the original name, not the alias
        assert_eq!(imports[0].imported_symbols, vec!["useState"]);
    }

    #[test]
    fn test_multiple_imports_same_line() {
        let source = r#"import A from 'a'; import B from 'b';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 2);
        assert!(imports.iter().any(|i| i.package_name == "a"));
        assert!(imports.iter().any(|i| i.package_name == "b"));
    }

    #[test]
    fn test_import_with_subpath() {
        let source = r#"import { thing } from 'package/subpath';"#;
        let imports = analyze_js(source);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].package_name, "package/subpath");
    }
}
