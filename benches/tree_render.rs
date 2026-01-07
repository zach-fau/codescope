//! Benchmarks for tree rendering performance
//!
//! Tests rendering performance with large dependency trees to ensure
//! smooth 60fps scrolling with 1000+ nodes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use codescope::ui::tree::TreeNode;

/// Create a large test tree with specified number of nodes
fn create_large_tree(total_nodes: usize, max_depth: usize, children_per_node: usize) -> TreeNode {
    let mut root = TreeNode::new("root".to_string(), "1.0.0".to_string());
    root.expanded = true;

    let mut node_count = 1;

    fn add_children(
        parent: &mut TreeNode,
        node_count: &mut usize,
        total_nodes: usize,
        current_depth: usize,
        max_depth: usize,
        children_per_node: usize,
    ) {
        if *node_count >= total_nodes || current_depth >= max_depth {
            return;
        }

        for i in 0..children_per_node {
            if *node_count >= total_nodes {
                break;
            }

            let mut child = TreeNode::new(
                format!("dep-{}-{}", current_depth, i),
                format!("{}.0.0", *node_count),
            );
            child.expanded = true;
            *node_count += 1;

            add_children(
                &mut child,
                node_count,
                total_nodes,
                current_depth + 1,
                max_depth,
                children_per_node,
            );

            parent.add_child(child);
        }
    }

    add_children(&mut root, &mut node_count, total_nodes, 1, max_depth, children_per_node);
    root
}

/// Benchmark tree flattening operation
fn bench_flatten(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_flatten");

    for size in [100, 500, 1000, 2000, 5000].iter() {
        let tree = create_large_tree(*size, 10, 5);

        group.bench_with_input(BenchmarkId::new("nodes", size), size, |b, _| {
            b.iter(|| {
                black_box(tree.flatten())
            });
        });
    }

    group.finish();
}

/// Benchmark tree prefix calculation (for rendering)
fn bench_tree_prefix(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_prefix");

    for size in [100, 500, 1000, 2000].iter() {
        let tree = create_large_tree(*size, 10, 5);
        let flattened = tree.flatten();

        group.bench_with_input(BenchmarkId::new("nodes", size), &flattened, |b, flat| {
            b.iter(|| {
                // Simulate rendering all visible items
                for node in flat.iter() {
                    black_box(node.expansion_indicator());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark virtual scrolling simulation (only process visible nodes)
fn bench_virtual_scroll(c: &mut Criterion) {
    let mut group = c.benchmark_group("virtual_scroll");

    // Typical terminal height is around 40-50 rows
    let visible_rows = 50;

    for size in [1000, 2000, 5000, 10000].iter() {
        let tree = create_large_tree(*size, 10, 5);
        let flattened = tree.flatten();

        group.bench_with_input(
            BenchmarkId::new("nodes_visible_50", size),
            &flattened,
            |b, flat| {
                b.iter(|| {
                    // Only process visible nodes (virtual scrolling)
                    let start = 0;
                    let end = visible_rows.min(flat.len());
                    for node in flat[start..end].iter() {
                        black_box(node.expansion_indicator());
                        black_box(&node.name);
                        black_box(&node.version);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark toggle operation at various tree positions
fn bench_toggle(c: &mut Criterion) {
    let mut group = c.benchmark_group("toggle_node");

    for size in [500, 1000, 2000].iter() {
        let mut tree = create_large_tree(*size, 10, 5);

        group.bench_with_input(BenchmarkId::new("nodes", size), size, |b, _| {
            b.iter(|| {
                // Toggle a node in the middle
                let middle = *size / 2;
                tree.toggle_at_index(middle);
                tree.toggle_at_index(middle); // Toggle back
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_flatten,
    bench_tree_prefix,
    bench_virtual_scroll,
    bench_toggle
);
criterion_main!(benches);
