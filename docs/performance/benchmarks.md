# Benchmarks

rapid_textrank achieves significant speedups over pure Python TextRank implementations through Rust's performance characteristics and careful algorithm implementation.

## Pre-computed Results

The table below shows approximate timings measured on a modern laptop. Your results will vary depending on hardware, Python version, and system load.

| Document Size | rapid_textrank | pytextrank + spaCy | Speedup |
|---|---|---|---|
| Small (~20 words) | ~0.1 ms | ~5 ms | ~50x |
| Medium (~100 words) | ~0.3 ms | ~15 ms | ~50x |
| Large (~1000 words) | ~2 ms | ~80 ms | ~40x |

!!! note "About these numbers"
    Results are approximate and depend on hardware. Run the benchmark script below to measure on your system.

## Benchmark Script

Use the script below to compare rapid_textrank and pytextrank performance on your own hardware.

??? note "Benchmark Script"
    ```python
    """
    Benchmark: rapid_textrank vs pytextrank

    Prerequisites:
        pip install rapid_textrank pytextrank spacy
        python -m spacy download en_core_web_sm
    """

    import time
    import statistics

    # Sample texts of varying sizes
    TEXTS = {
        "small": """
            Machine learning is a subset of artificial intelligence.
            Deep learning uses neural networks with many layers.
        """,

        "medium": """
            Natural language processing (NLP) is a field of artificial intelligence
            that focuses on the interaction between computers and humans through
            natural language. The ultimate goal of NLP is to enable computers to
            understand, interpret, and generate human language in a valuable way.

            Machine learning approaches have transformed NLP in recent years.
            Deep learning models, particularly transformers, have achieved
            state-of-the-art results on many NLP tasks including translation,
            summarization, and question answering.

            Key applications include sentiment analysis, named entity recognition,
            machine translation, and text classification. These technologies
            power virtual assistants, search engines, and content recommendation
            systems used by millions of people daily.
        """,

        "large": """
            Artificial intelligence has evolved dramatically since its inception in
            the mid-20th century. Early AI systems relied on symbolic reasoning and
            expert systems, where human knowledge was manually encoded into rules.

            The machine learning revolution changed everything. Instead of explicit
            programming, systems learn patterns from data. Supervised learning uses
            labeled examples, unsupervised learning finds hidden structures, and
            reinforcement learning optimizes through trial and error.

            Deep learning, powered by neural networks with multiple layers, has
            achieved remarkable success. Convolutional neural networks excel at
            image recognition. Recurrent neural networks and transformers handle
            sequential data like text and speech. Generative adversarial networks
            create realistic synthetic content.

            Natural language processing has been transformed by these advances.
            Word embeddings capture semantic relationships. Attention mechanisms
            allow models to focus on relevant context. Large language models
            demonstrate emergent capabilities in reasoning and generation.

            Computer vision applications include object detection, facial recognition,
            medical image analysis, and autonomous vehicle perception. These systems
            process visual information with superhuman accuracy in many domains.

            The ethical implications of AI are significant. Bias in training data
            can lead to unfair outcomes. Privacy concerns arise from data collection.
            Job displacement affects workers across industries. Regulation and
            governance frameworks are being developed worldwide.

            Future directions include neuromorphic computing, quantum machine learning,
            and artificial general intelligence. Researchers continue to push
            boundaries while addressing safety and alignment challenges.
        """ * 3  # ~1000 words
    }


    def benchmark_rapid_textrank(text: str, runs: int = 10) -> dict:
        """Benchmark rapid_textrank."""
        from rapid_textrank import BaseTextRank

        extractor = BaseTextRank(top_n=10, language="en")

        # Warmup
        extractor.extract_keywords(text)

        times = []
        for _ in range(runs):
            start = time.perf_counter()
            result = extractor.extract_keywords(text)
            elapsed = time.perf_counter() - start
            times.append(elapsed * 1000)  # Convert to ms

        return {
            "min": min(times),
            "mean": statistics.mean(times),
            "median": statistics.median(times),
            "std": statistics.stdev(times) if len(times) > 1 else 0,
            "phrases": len(result.phrases)
        }


    def benchmark_pytextrank(text: str, runs: int = 10) -> dict:
        """Benchmark pytextrank with spaCy."""
        import spacy
        import pytextrank

        nlp = spacy.load("en_core_web_sm")
        nlp.add_pipe("textrank")

        # Warmup
        doc = nlp(text)

        times = []
        for _ in range(runs):
            start = time.perf_counter()
            doc = nlp(text)
            phrases = list(doc._.phrases[:10])
            elapsed = time.perf_counter() - start
            times.append(elapsed * 1000)

        return {
            "min": min(times),
            "mean": statistics.mean(times),
            "median": statistics.median(times),
            "std": statistics.stdev(times) if len(times) > 1 else 0,
            "phrases": len(phrases)
        }


    def main():
        print("=" * 70)
        print("TextRank Performance Benchmark")
        print("=" * 70)

        for size, text in TEXTS.items():
            word_count = len(text.split())
            print(f"\n{size.upper()} TEXT (~{word_count} words)")
            print("-" * 50)

            # Benchmark rapid_textrank
            rust_results = benchmark_rapid_textrank(text)
            print(f"rapid_textrank:  {rust_results['mean']:>8.2f} ms (±{rust_results['std']:.2f})")

            # Benchmark pytextrank
            try:
                py_results = benchmark_pytextrank(text)
                print(f"pytextrank:     {py_results['mean']:>8.2f} ms (±{py_results['std']:.2f})")

                speedup = py_results['mean'] / rust_results['mean']
                print(f"Speedup:        {speedup:>8.1f}x faster")
            except Exception as e:
                print(f"pytextrank:     (not available: {e})")

        print("\n" + "=" * 70)
        print("Note: pytextrank times include spaCy tokenization.")
        print("For fair comparison with pre-tokenized input, use rapid_textrank's JSON API.")
        print("=" * 70)


    if __name__ == "__main__":
        main()
    ```

## Notes on Fair Comparison

pytextrank times include spaCy tokenization (loading the pipeline, running the tokenizer, POS tagger, lemmatizer, etc.). For a fair comparison with pre-tokenized input, use rapid_textrank's JSON API, which accepts tokens that have already been processed by spaCy or another NLP pipeline.

When comparing end-to-end latency (raw text in, keywords out), the rapid_textrank native classes include a built-in tokenizer that is much lighter than spaCy's full pipeline. This accounts for a significant portion of the observed speedup.

!!! tip "Interactive Notebook"
    Run benchmarks interactively in the [Benchmarks Notebook](https://github.com/xang1234/rapid-textrank/blob/main/notebooks/04_benchmarks.ipynb).
