"""
Benchmark: batch API comparison — sequential vs batch vs iterator (workspace reuse).

Compares three JSON API modes:
  1. Sequential  — loop of extract_from_json() calls, one doc at a time
  2. Batch array — extract_batch_from_json() with JSON array, returns one big string
  3. Batch iter  — extract_batch_iter() with workspace reuse, yields per-doc strings

Measures:
  - Total wall-clock time (mean ± stdev across rounds)
  - Per-document throughput (docs/sec)
  - Peak memory (via tracemalloc)

Usage:
    python python/benchmarks/bench_batch_comparison.py [--docs 200] [--rounds 5]
"""

from __future__ import annotations

import argparse
import gc
import json
import statistics
import time
import tracemalloc

from rapid_textrank import extract_batch_from_json, extract_batch_iter, extract_from_json

# ---------------------------------------------------------------------------
# Document generation — pre-tokenized JSON docs (what the JSON API expects)
# ---------------------------------------------------------------------------

_SENTENCE_TEMPLATES = [
    [
        ("Machine", "machine", "NOUN"),
        ("learning", "learning", "NOUN"),
        ("algorithms", "algorithm", "NOUN"),
        ("process", "process", "VERB"),
        ("large", "large", "ADJ"),
        ("datasets", "dataset", "NOUN"),
    ],
    [
        ("Natural", "natural", "ADJ"),
        ("language", "language", "NOUN"),
        ("processing", "processing", "NOUN"),
        ("enables", "enable", "VERB"),
        ("text", "text", "NOUN"),
        ("understanding", "understanding", "NOUN"),
    ],
    [
        ("Neural", "neural", "ADJ"),
        ("networks", "network", "NOUN"),
        ("form", "form", "VERB"),
        ("the", "the", "DET"),
        ("backbone", "backbone", "NOUN"),
        ("of", "of", "ADP"),
        ("deep", "deep", "ADJ"),
        ("learning", "learning", "NOUN"),
    ],
    [
        ("Gradient", "gradient", "NOUN"),
        ("descent", "descent", "NOUN"),
        ("optimizes", "optimize", "VERB"),
        ("model", "model", "NOUN"),
        ("parameters", "parameter", "NOUN"),
        ("during", "during", "ADP"),
        ("training", "training", "NOUN"),
    ],
    [
        ("Feature", "feature", "NOUN"),
        ("engineering", "engineering", "NOUN"),
        ("transforms", "transform", "VERB"),
        ("raw", "raw", "ADJ"),
        ("data", "data", "NOUN"),
        ("into", "into", "ADP"),
        ("useful", "useful", "ADJ"),
        ("representations", "representation", "NOUN"),
    ],
    [
        ("Convolutional", "convolutional", "ADJ"),
        ("neural", "neural", "ADJ"),
        ("networks", "network", "NOUN"),
        ("excel", "excel", "VERB"),
        ("at", "at", "ADP"),
        ("image", "image", "NOUN"),
        ("classification", "classification", "NOUN"),
    ],
    [
        ("Distributed", "distributed", "ADJ"),
        ("systems", "system", "NOUN"),
        ("coordinate", "coordinate", "VERB"),
        ("multiple", "multiple", "ADJ"),
        ("computing", "computing", "NOUN"),
        ("nodes", "node", "NOUN"),
    ],
    [
        ("Database", "database", "NOUN"),
        ("optimization", "optimization", "NOUN"),
        ("improves", "improve", "VERB"),
        ("query", "query", "NOUN"),
        ("performance", "performance", "NOUN"),
        ("through", "through", "ADP"),
        ("indexing", "indexing", "NOUN"),
    ],
]

# Stopwords to mark
_STOPWORDS = {"the", "of", "at", "into", "during", "through"}


def _build_tokens(num_sentences: int) -> list[dict]:
    """Build a token list with `num_sentences` sentences (cycling templates)."""
    tokens = []
    char_offset = 0
    token_idx = 0
    for sent_idx in range(num_sentences):
        template = _SENTENCE_TEMPLATES[sent_idx % len(_SENTENCE_TEMPLATES)]
        for text, lemma, pos in template:
            end = char_offset + len(text)
            tokens.append({
                "text": text,
                "lemma": lemma,
                "pos": pos,
                "start": char_offset,
                "end": end,
                "sentence_idx": sent_idx,
                "token_idx": token_idx,
                "is_stopword": lemma.lower() in _STOPWORDS,
            })
            char_offset = end + 1
            token_idx += 1
    return tokens


def generate_docs(
    n: int,
    *,
    sentences_per_doc: int = 8,
    use_pipeline: bool = True,
) -> list[dict]:
    """Generate n JSON documents (as dicts). Cycles sentence count for variety."""
    docs = []
    for i in range(n):
        sents = sentences_per_doc + (i % 4) * 2  # 8, 10, 12, 14 sentences
        doc: dict = {
            "tokens": _build_tokens(sents),
            "config": {"top_n": 10, "determinism": "deterministic"},
        }
        if use_pipeline:
            doc["pipeline"] = "textrank"
        docs.append(doc)
    return docs


# ---------------------------------------------------------------------------
# Benchmark runners
# ---------------------------------------------------------------------------


def bench_sequential(docs_json: list[str]) -> list[str]:
    """Call extract_from_json() once per document. Return results."""
    results = []
    for d in docs_json:
        results.append(extract_from_json(d))
    return results


def bench_batch_array(batch_json: str) -> list:
    """Call extract_batch_from_json() with the entire array. Parse result."""
    result_str = extract_batch_from_json(batch_json)
    return json.loads(result_str)


def bench_batch_iter(batch_json: str) -> list[str]:
    """Call extract_batch_iter() and consume the iterator."""
    return list(extract_batch_iter(batch_json))


# ---------------------------------------------------------------------------
# Memory measurement
# ---------------------------------------------------------------------------


def measure_peak_memory(fn) -> tuple[float, float]:
    """Run fn(), return (wall_time, peak_memory_bytes)."""
    gc.collect()
    tracemalloc.start()
    t0 = time.perf_counter()
    fn()
    wall = time.perf_counter() - t0
    _, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    return wall, peak


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------


def fmt_ms(seconds: float) -> str:
    return f"{seconds * 1000:.2f} ms"


def fmt_mem(bytes_: float) -> str:
    if bytes_ >= 1024 * 1024:
        return f"{bytes_ / 1024 / 1024:.2f} MB"
    if bytes_ >= 1024:
        return f"{bytes_ / 1024:.1f} KB"
    return f"{bytes_:.0f} B"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description="Benchmark batch vs sequential")
    parser.add_argument("--docs", type=int, default=200, help="Number of documents (default: 200)")
    parser.add_argument("--warmup", type=int, default=2, help="Warmup rounds (default: 2)")
    parser.add_argument("--rounds", type=int, default=5, help="Measurement rounds (default: 5)")
    parser.add_argument(
        "--sentences", type=int, default=8,
        help="Base sentences per document (default: 8)",
    )
    parser.add_argument(
        "--variant", action="store_true",
        help="Use legacy variant path instead of pipeline path",
    )
    args = parser.parse_args()

    use_pipeline = not args.variant
    path_label = "pipeline" if use_pipeline else "variant (legacy)"

    docs = generate_docs(args.docs, sentences_per_doc=args.sentences, use_pipeline=use_pipeline)
    docs_json = [json.dumps(d) for d in docs]
    batch_json = json.dumps(docs)

    avg_tokens = sum(len(d["tokens"]) for d in docs) / len(docs)

    print("Batch Comparison Benchmark")
    print(f"  Documents    : {args.docs}")
    print(f"  Path         : {path_label}")
    print(f"  Avg tokens   : {avg_tokens:.0f} per doc")
    print(f"  Rounds       : {args.rounds} (+ {args.warmup} warmup)")
    print(f"  Batch JSON   : {len(batch_json) / 1024:.1f} KB")

    # ── Warmup ────────────────────────────────────────────────────────
    for _ in range(args.warmup):
        bench_sequential(docs_json)
        bench_batch_array(batch_json)
        bench_batch_iter(batch_json)

    # ── Time measurement ──────────────────────────────────────────────
    seq_times: list[float] = []
    batch_times: list[float] = []
    iter_times: list[float] = []

    for _ in range(args.rounds):
        t0 = time.perf_counter()
        bench_sequential(docs_json)
        seq_times.append(time.perf_counter() - t0)

        t0 = time.perf_counter()
        bench_batch_array(batch_json)
        batch_times.append(time.perf_counter() - t0)

        t0 = time.perf_counter()
        bench_batch_iter(batch_json)
        iter_times.append(time.perf_counter() - t0)

    # ── Memory measurement (single run) ───────────────────────────────
    _, seq_peak = measure_peak_memory(lambda: bench_sequential(docs_json))
    _, batch_peak = measure_peak_memory(lambda: bench_batch_array(batch_json))
    _, iter_peak = measure_peak_memory(lambda: bench_batch_iter(batch_json))

    # ── Report ────────────────────────────────────────────────────────
    def show(label: str, times: list[float], peak_mem: float) -> float:
        mean = statistics.mean(times)
        std = statistics.stdev(times) if len(times) > 1 else 0.0
        throughput = args.docs / mean
        print(f"\n{'─' * 60}")
        print(f"  {label}")
        print(f"{'─' * 60}")
        print(f"  Wall clock    : {fmt_ms(mean)} ± {fmt_ms(std)}")
        print(f"  Throughput    : {throughput:,.0f} docs/sec")
        print(f"  Peak memory   : {fmt_mem(peak_mem)}")
        return mean

    seq_mean = show("1. Sequential (extract_from_json loop)", seq_times, seq_peak)
    batch_mean = show("2. Batch array (extract_batch_from_json)", batch_times, batch_peak)
    iter_mean = show("3. Batch iter  (extract_batch_iter + workspace reuse)", iter_times, iter_peak)

    # ── Summary table ─────────────────────────────────────────────────
    print(f"\n{'━' * 60}")
    print(f"  {'Mode':<40} {'Speedup':>8}  {'Memory':>10}")
    print(f"  {'─' * 40} {'─' * 8}  {'─' * 10}")
    for label, mean, peak in [
        ("Sequential (baseline)", seq_mean, seq_peak),
        ("Batch array", batch_mean, batch_peak),
        ("Batch iter (workspace reuse)", iter_mean, iter_peak),
    ]:
        sp = seq_mean / mean if mean > 0 else 0
        mem_ratio = peak / seq_peak if seq_peak > 0 else 0
        print(f"  {label:<40} {sp:>7.2f}x  {mem_ratio:>9.1f}x")
    print(f"{'━' * 60}")

    # ── Workspace reuse benefit ───────────────────────────────────────
    if iter_mean < batch_mean:
        pct = (1 - iter_mean / batch_mean) * 100
        print(f"\n  Workspace reuse advantage: {pct:.1f}% faster than batch array")
    else:
        pct = (iter_mean / batch_mean - 1) * 100
        print(f"\n  Workspace reuse overhead: {pct:.1f}% slower than batch array")

    if iter_peak < batch_peak:
        pct = (1 - iter_peak / batch_peak) * 100
        print(f"  Memory savings: {pct:.1f}% less peak memory than batch array")


if __name__ == "__main__":
    main()
