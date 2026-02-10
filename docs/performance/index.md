# Performance

rapid_textrank is designed for speed. The Rust core delivers **10-100x faster** keyword extraction compared to pure Python implementations, depending on document size and tokenization method.

## Key Performance Features

- **Rust core with zero-copy data paths** -- most computation happens in compiled Rust code, minimizing Python overhead
- **CSR graph format** -- Compressed Sparse Row storage for cache-friendly PageRank iteration
- **String interning** -- `StringPool` reduces memory allocations 10-100x for typical documents
- **Parallel processing** -- Rayon provides data parallelism for internal graph construction
- **Link-Time Optimization** -- full LTO with single codegen unit for maximum inlining
- **FxHash** -- fast non-cryptographic hashing for internal hash maps

## Approximate Speedups

| Document Size | rapid_textrank | pytextrank + spaCy | Speedup |
|---|---|---|---|
| Small (~20 words) | ~0.1 ms | ~5 ms | ~50x |
| Medium (~100 words) | ~0.3 ms | ~15 ms | ~50x |
| Large (~1000 words) | ~2 ms | ~80 ms | ~40x |

Results are approximate and vary by hardware. See the [Benchmarks](benchmarks.md) page for a runnable benchmark script.

## Learn More

- **[Benchmarks](benchmarks.md)** -- detailed benchmark results and a script to measure performance on your system
- **[Why Rust is Fast](why-rust-is-fast.md)** -- deep dive into the performance optimizations used in rapid_textrank
- **[Comparison](comparison.md)** -- how rapid_textrank compares to alternative keyword extraction libraries
