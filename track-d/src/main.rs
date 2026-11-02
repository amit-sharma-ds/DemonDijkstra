use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

struct FastInput { bytes: Vec<u8>, pos: usize }

impl FastInput {
    fn from_file(path: &str) -> io::Result<Self> {
        Ok(Self { bytes: fs::read(path)?, pos: 0 })
    }

    // Parse the known ASCII decimal format directly, without line/string allocations.
    fn next_u32(&mut self) -> u32 {
        while self.pos < self.bytes.len() && self.bytes[self.pos] <= b' ' { self.pos += 1; }
        let mut value = 0u32;
        while self.pos < self.bytes.len() {
            let byte = self.bytes[self.pos];
            if !byte.is_ascii_digit() { break; }
            value = value * 10 + u32::from(byte - b'0');
            self.pos += 1;
        }
        value
    }

    fn has_next(&mut self) -> bool {
        while self.pos < self.bytes.len() && self.bytes[self.pos] <= b' ' { self.pos += 1; }
        self.pos < self.bytes.len()
    }
}

struct Csr { offsets: Vec<usize>, heads: Vec<u32>, weights: Vec<u32>, has_in_edge: Vec<u8> }

impl Csr {
    fn read(path: &str) -> io::Result<(usize, Self)> {
        let mut input = FastInput::from_file(path)?;
        let n = input.next_u32() as usize;
        let m = input.next_u32() as usize;
        let mut edges = Vec::with_capacity(m);
        let mut degree = vec![0usize; n];
        let mut has_in_edge = vec![0u8; n];
        for _ in 0..m {
            let u = input.next_u32();
            let v = input.next_u32();
            let w = input.next_u32();
            degree[u as usize] += 1;
            has_in_edge[v as usize] = 1;
            edges.push((u, v, w));
        }

        // Prefix sums make each node's outgoing edges a contiguous CSR range.
        // Keeping parallel/duplicate edges is correct; relaxations discard losers.
        let mut offsets = vec![0usize; n + 1];
        for u in 0..n { offsets[u + 1] = offsets[u] + degree[u]; }
        let mut cursor = offsets[..n].to_vec();
        let mut heads = vec![0u32; m];
        let mut weights = vec![0u32; m];
        for (u, v, w) in edges {
            let slot = cursor[u as usize];
            heads[slot] = v;
            weights[slot] = w;
            cursor[u as usize] += 1;
        }
        Ok((n, Self { offsets, heads, weights, has_in_edge }))
    }
}

struct Scratch { dist: Vec<u64>, stamp: Vec<u32>, epoch: u32, heap: BinaryHeap<Reverse<(u64, u32)>> }

impl Scratch {
    fn new(n: usize) -> Self {
        Self { dist: vec![0; n], stamp: vec![0; n], epoch: 0, heap: BinaryHeap::new() }
    }

    fn shortest_path(&mut self, graph: &Csr, source: u32, target: u32) -> Option<u64> {
        if source == target { return Some(0); }
        // These structural checks avoid a full search for impossible paths.
        // A non-source target with no incoming edge is unreachable by definition.
        if graph.has_in_edge[target as usize] == 0 { return None; }
        let source_index = source as usize;
        if graph.offsets[source_index] == graph.offsets[source_index + 1] { return None; }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            // Clear generations only on the practically unreachable 32-bit wrap.
            self.stamp.fill(0);
            self.epoch = 1;
        }
        let epoch = self.epoch;
        self.heap.clear();
        self.dist[source as usize] = 0;
        self.stamp[source as usize] = epoch;
        self.heap.push(Reverse((0, source)));
        while let Some(Reverse((distance, u))) = self.heap.pop() {
            let ui = u as usize;
            if self.stamp[ui] != epoch || self.dist[ui] != distance { continue; }
            if u == target { return Some(distance); }
            for edge in graph.offsets[ui]..graph.offsets[ui + 1] {
                let v = graph.heads[edge];
                let candidate = distance + u64::from(graph.weights[edge]);
                let vi = v as usize;
                if self.stamp[vi] != epoch || candidate < self.dist[vi] {
                    self.stamp[vi] = epoch;
                    self.dist[vi] = candidate;
                    self.heap.push(Reverse((candidate, v)));
                }
            }
        }
        None
    }
}

fn main() -> io::Result<()> {
    let (n, graph) = Csr::read("/data/graph.txt")?;
    let mut query_input = FastInput::from_file("/data/queries.txt")?;
    let mut queries = Vec::new();
    while query_input.has_next() {
        queries.push((query_input.next_u32(), query_input.next_u32()));
    }

    let workers = thread::available_parallelism().map(|v| v.get()).unwrap_or(1).min(12).min(queries.len().max(1));
    let next = Arc::new(AtomicUsize::new(0));
    let queries = Arc::new(queries);
    let graph = Arc::new(graph);
    let mut joins = Vec::with_capacity(workers);
    for _ in 0..workers {
        let next = Arc::clone(&next);
        let queries = Arc::clone(&queries);
        let graph = Arc::clone(&graph);
        joins.push(thread::spawn(move || {
            let mut scratch = Scratch::new(n);
            let mut answers = Vec::new();
            loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= queries.len() { break; }
                let (source, target) = queries[index];
                answers.push((index, scratch.shortest_path(&graph, source, target)));
            }
            answers
        }));
    }
    let mut answers = vec![None; queries.len()];
    for join in joins {
        for (index, answer) in join.join().expect("worker thread panicked") { answers[index] = Some(answer); }
    }

    fs::create_dir_all("/out")?;
    let mut output = BufWriter::new(fs::File::create("/out/distances.txt")?);
    for answer in answers {
        match answer.expect("every query is assigned") {
            Some(distance) => writeln!(output, "{distance}")?,
            None => writeln!(output, "-1")?,
        }
    }
    output.flush()
}
