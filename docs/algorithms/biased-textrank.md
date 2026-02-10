# BiasedTextRank

BiasedTextRank (Kazemi et al., 2020) steers keyword extraction toward specific topics by biasing the PageRank random walk toward user-specified focus terms. The `bias_weight` parameter controls how strongly results favor those terms -- higher values produce results more tightly clustered around the focus vocabulary.

## How It Works

BiasedTextRank modifies the PageRank personalization vector so that focus terms receive a higher teleport probability. Non-focus words still participate in the graph, but the random walk preferentially returns to the focus terms, boosting them and their neighbors in the final ranking.

The co-occurrence graph construction and phrase extraction steps are the same as [BaseTextRank](base-textrank.md).

## Usage

```python
from rapid_textrank import BiasedTextRank

extractor = BiasedTextRank(
    focus_terms=["security", "privacy"],
    bias_weight=5.0,  # Higher = stronger bias
    top_n=10
)

result = extractor.extract_keywords("""
Modern web applications must balance user experience with security.
Privacy regulations require careful data handling. Performance
optimizations should not compromise security measures.
""")

# Results will favor security/privacy-related phrases
```

### Per-call focus override

You can reuse the same extractor instance with different focus terms for each call:

```python
result = extractor.extract_keywords(text, focus_terms=["neural", "network"])
```

This is useful in pipelines where the extractor configuration (window size, POS filters, etc.) stays the same but the focus topic changes per document or per query.

### Choosing bias_weight

- Values in the range **3.0 -- 10.0** work well for most use cases.
- Lower values (1.0 -- 3.0) produce a gentle nudge -- results still include diverse terms.
- Higher values (10.0+) strongly concentrate results around the focus terms and their immediate neighbors.

## When to Use BiasedTextRank

BiasedTextRank is a good fit when you know the domain or topic you care about and can express it as a short list of terms:

- **Security audits** -- focus on `["encrypt", "tls", "mfa", "audit", "privacy"]`.
- **Query-driven extraction** -- set focus terms from user input (e.g., a search query).
- **Domain-specific pipelines** -- pre-define focus vocabularies for legal, medical, or financial text.

If you have per-word importance scores (e.g., from a topic model) rather than a flat term list, consider [TopicalPageRank](topical-pagerank.md) instead.

## Reference

- [BiasedTextRank: Unsupervised Graph-Based Content Extraction](https://aclanthology.org/2020.coling-main.144/) (Kazemi et al., 2020)
