# DemonDijkstra

Millisecond-fast shortest paths on 5M roads.

Track D: Traverse computes directed shortest-path distances for each query.

## Approach

The binary reads the graph and queries into memory with a hand-written byte scanner. It counts outgoing edges, prefix-sums those counts, and scatters edges into compressed sparse row (CSR) storage. Each query runs Dijkstra with a binary min-heap and stops when the target is settled. Distances use `u64`; generation-stamped distance arrays avoid clearing all nodes between searches. Up to 12 standard-library threads dynamically take queries, and results are emitted in input order.

Parallel and duplicate edges, loops, zero weights, disconnected nodes, and duplicate queries are supported directly. No graph data is precomputed.

## Dependencies

None. Build with `cargo build --release --offline`.

## With another 24 hours

Benchmark on the live scoring board and compare heap choices and worker scheduling. Profile full-scale peak memory, then test a radix heap or per-source reuse only where measurements show a clear improvement.
