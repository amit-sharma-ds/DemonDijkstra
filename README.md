# DemonDijkstra

**Millisecond-fast shortest paths on 5M roads.**

DemonDijkstra is a zero-dependency Rust solution for Speed Demon 2026 Track D: Traverse. It reads a directed weighted graph and a list of source/target queries, then writes one exact shortest-path distance per query.

## Project structure

```text
DemonDijkstra/
├── Cargo.toml          # Package and release build settings
├── Dockerfile          # Offline, pinned multi-stage container build
├── LICENSE             # MIT
├── PERF.md             # Benchmark history and board run IDs
├── README.md
└── src/
    └── main.rs         # Parser, CSR graph, Dijkstra, worker pool, output
```

The contest branch keeps the same project under `track-d/`, as required by the Speed Demon harness.

## How it works

1. **Parse once:** a small byte scanner reads the ASCII inputs without allocating a string for every line.
2. **Build CSR:** count each node's outgoing edges, prefix-sum the counts, then scatter edge heads and `u32` weights into contiguous arrays. Parallel and exact duplicate edges remain valid CSR entries; Dijkstra ignores any relaxation that does not improve a distance.
3. **Search per query:** binary-heap Dijkstra uses `u64` distances, skips stale heap entries, and stops as soon as the target is removed from the heap. A generation stamp marks which distance entries belong to the current search, avoiding an `O(n)` reset per query.
4. **Use available cores:** up to 12 standard-library workers claim queries dynamically. Each worker owns its distance/stamp arrays and heap; answers are placed by query index and written in the original order.

Before searching, two general graph checks return immediately when the source has no outgoing edges or the distinct target has no incoming edges. Both cases make a path impossible.

The approach handles zero-weight edges, self-loops, directed one-way roads, disconnected nodes, unreachable targets, repeated queries, and path totals above `2^32 - 1`. It does not use a shortest-path crate or precompute answers from a dataset.

## Build

```sh
cargo build --release --offline
```

The executable is `target/release/demondijkstra`. The Dockerfile builds it offline from the pinned Rust image and runs it in the pinned Debian image.

## Runtime contract

The binary reads `/data/graph.txt` and `/data/queries.txt`, then writes only `/out/distances.txt`. Each output line is the exact distance as a decimal integer, or `-1` when the target is unreachable. The output has one line per query and ends with a newline.

## Correctness checks

The Track D worked example and sample dataset hashes are specified in the challenge. For repeatable local checks, generate the sample with the official `d_traverse.py` generator and run the official `check.py d` harness with the expected sample SHA-256. The expected `distances.txt` hash is:

```text
a4c76ed030bc7e365db33191f6cf74708fba7f25eb48d64c8c4279f2e57046a0
```

## Performance work

The core optimizations are whole-file byte parsing, compact CSR storage, graph-invariant early rejection, early-exit Dijkstra, generation-stamped scratch arrays, and parallel query execution. `PERF.md` records measured changes; timings and rank should only be added after an actual board run. A README cannot guarantee first place—the score depends on correctness, the measured full-data runtime, memory use, and other teams' submissions.

For the contest, submit the repository and Track D entry, install the board app, and push the `speed-demon` branch. Each push can trigger a board run; record its real run ID and metrics in `track-d/PERF.md`.

## With another 24 hours

Use board timings and profiling to test a radix heap, query scheduling, and alternative CSR construction independently. Keep a change only when repeated board runs show a real gain and the exact-output check still passes.
