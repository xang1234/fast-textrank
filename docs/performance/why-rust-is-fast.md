# Why Rust is Fast

The performance advantage of rapid_textrank comes from several factors working together. Each optimization targets a specific bottleneck in the TextRank pipeline.

## 1. CSR Graph Format

The co-occurrence graph uses **Compressed Sparse Row (CSR)** format, which stores all edge data in contiguous arrays rather than pointer-chasing through linked structures.

Why it matters: PageRank iterates over every node's neighbors many times (typically 20-100 iterations until convergence). CSR format keeps each node's neighbors in adjacent memory locations, which means the CPU cache can prefetch the next edge while processing the current one. This cache-friendly access pattern avoids the random memory access penalties that adjacency-list or dictionary-of-dictionaries representations suffer from, especially as graph size grows.

## 2. String Interning

Repeated words share a single allocation via `StringPool`, reducing memory usage **10-100x** for typical documents.

Why it matters: Natural language text is highly repetitive. A 1,000-word document might contain only 200-300 unique words, but a naive implementation allocates a new string for every occurrence. String interning maps each unique word to a small integer ID, so the graph, co-occurrence counts, and PageRank vectors all operate on compact integer keys rather than heap-allocated strings. This reduces both memory pressure and the cost of hash lookups and equality comparisons throughout the pipeline.

## 3. Parallel Processing

**Rayon** provides data parallelism for internal graph construction without explicit thread management.

Why it matters: Building the co-occurrence graph involves scanning every sliding window in the document and updating edge weights. Rayon's work-stealing scheduler automatically distributes this work across available CPU cores. For large documents or batch processing, this parallelism translates directly into wall-clock speedups proportional to core count. The programmer writes sequential-looking iterators, and Rayon handles the threading, synchronization, and load balancing.

## 4. Link-Time Optimization (LTO)

Release builds use **full LTO with a single codegen unit** for maximum inlining.

Why it matters: By default, the Rust compiler splits code into multiple codegen units for faster compilation, but this prevents cross-unit inlining. Full LTO merges all code into a single compilation unit, allowing the optimizer to inline small functions across module boundaries, eliminate dead code, and perform whole-program optimizations. For rapid_textrank, this means hot loops in PageRank iteration, graph construction, and string interning can be fully inlined and optimized as a single block of machine code.

## 5. Rust Core

Most computation happens in compiled Rust code, **minimizing Python-level overhead**.

Why it matters: Python's interpreter adds overhead for every operation -- attribute lookups, dynamic dispatch, reference counting, and the Global Interpreter Lock (GIL). By implementing the core algorithm in Rust and exposing only the entry/exit points to Python via PyO3, rapid_textrank avoids this overhead for the performance-critical inner loops. The Python layer handles configuration and result formatting, while the Rust layer handles graph construction, PageRank iteration, and phrase extraction -- the three most compute-intensive steps.

## 6. FxHash

**Fast non-cryptographic hashing** for internal hash maps.

Why it matters: The standard library's `HashMap` uses SipHash, which is designed to resist hash-flooding attacks. This security property is unnecessary for internal data structures where keys are controlled (word IDs, node indices). FxHash is a much simpler hash function that processes 4-8 bytes per cycle, making hash map lookups and insertions significantly faster. Since rapid_textrank's inner loops involve frequent hash map operations (co-occurrence counting, node lookups, score aggregation), switching to FxHash provides a measurable speedup.

## Combined Effect

These optimizations are not independent -- they compound. CSR format makes PageRank iteration cache-friendly, string interning makes the CSR arrays compact, FxHash makes the interning lookups fast, LTO lets the compiler optimize across all these layers, Rayon parallelizes the graph construction, and the Rust core avoids Python overhead for all of it. The result is an implementation that is typically 10-100x faster than equivalent pure Python code.
