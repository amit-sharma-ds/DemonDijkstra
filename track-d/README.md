# DemonDijkstra

**Millisecond-fast shortest paths on 5M roads.**

Zero-dependency Rust entry for Speed Demon 2026 Track D: Traverse. It reads a directed weighted graph and source/target queries, then writes the exact shortest distance for every query.

## Project structure

```text
track-d/
├── .speed-demon.toml  # Track, team, language, dependency count
├── Cargo.toml         # Rust package and release settings
├── Dockerfile         # Pinned offline build and runtime images
├── PERF.md            # Board run IDs, timings, and memory
├── README.md
└── src/
    └── main.rs        # Parser, CSR graph, Dijkstra, workers, output
```

## Approach

### Input and graph layout

A hand-written byte scanner parses the ASCII files without creating a string for each line. The graph is stored as compressed sparse row (CSR): outgoing-edge counts are prefix-summed into offsets, then heads and `u32` weights are scattered into contiguous arrays. Parallel edges and exact duplicates are kept; non-improving relaxations are ignored by Dijkstra.

### Shortest paths

Each query runs binary-heap Dijkstra from its source and exits when the target is popped with its settled distance. Distances are `u64`, so sums above `2^32 - 1` remain exact. A generation-stamped distance array marks values touched by the current query and avoids clearing all nodes before each search. Stale heap entries are skipped.

Before searching, the solver checks two graph invariants: a source with no outgoing edges cannot reach a different target, and a target with no incoming edges cannot be reached from a different source. Both return the correct answer without starting Dijkstra.

Up to 12 standard-library threads claim queries dynamically. Each worker owns its heap and scratch arrays. Results are placed in query-index order and written as a single `distances.txt` file.

The implementation supports zero-weight roads, self-loops, one-way roads, isolated nodes, unreachable targets, parallel edges, duplicate queries, and `s == t`. It does not use a shortest-path library or precomputed answers.

## Build and run

```sh
cargo build --release --offline
```

The binary is `target/release/demondijkstra`. The container reads `/data/graph.txt` and `/data/queries.txt` and writes only `/out/distances.txt`. Output lines follow query order and end with a newline.

## Correctness

Use the official worked example and `check.py d` against the generated sample data. The expected sample output hash is:

```text
a4c76ed030bc7e365db33191f6cf74708fba7f25eb48d64c8c4279f2e57046a0
```

## Performance plan

Parsing, CSR construction, graph-invariant early rejection, early-exit Dijkstra, stamped scratch space, and parallel queries are the current design. `PERF.md` is the source of truth for measured performance; add timings and run IDs only after the board returns them. First place depends on verified full-data runs and competing submissions, so the next step is to benchmark, inspect the slowest queries, and keep only optimizations that improve repeated runs without changing the output hash.

## With another 24 hours

Compare binary and radix heaps, measure CSR construction and query scheduling separately, and profile peak memory on the full graph. Validate each candidate with the exact-output checker before accepting its board result.
