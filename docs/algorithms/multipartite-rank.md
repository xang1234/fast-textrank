# MultipartiteRank

MultipartiteRank (Boudin, 2018) extends TopicRank by keeping individual candidates as graph nodes instead of collapsing topics into single representatives. It removes intra-topic edges to form a k-partite graph and applies an alpha weight adjustment that boosts the first-occurring variant in each topic cluster, encoding positional preference directly into edge weights.

## How It Works

1. **Candidate extraction** -- Candidate phrases are identified using POS-filtered noun chunks (same as other variants).
2. **Topic clustering** -- Candidates are grouped into topics based on Jaccard similarity over word sets, controlled by `similarity_threshold`.
3. **K-partite graph** -- A co-occurrence graph is built with all candidates as nodes, but edges between candidates in the same topic cluster are removed. This prevents intra-topic competition.
4. **Position boost** -- Edge weights are adjusted by an `alpha` factor that favors the first-occurring candidate in each topic cluster. Set `alpha=0` to disable this boost.
5. **PageRank** -- Standard PageRank is run on the k-partite graph.
6. **Phrase selection** -- Top-scoring candidates are returned directly (no topic-level aggregation step).

## Usage

### Native Python class

```python
from rapid_textrank import MultipartiteRank

extractor = MultipartiteRank(
    similarity_threshold=0.26,  # Jaccard threshold for topic clustering
    alpha=1.1,                  # Position boost strength (0 = disabled)
    top_n=10
)

result = extractor.extract_keywords("""
Machine learning is a powerful tool for data analysis. Deep learning
is a subset of machine learning. Neural networks power deep learning
systems. Convolutional neural networks excel at image recognition.
""")

for phrase in result.phrases:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

### JSON interface

MultipartiteRank is also available via the JSON interface with `variant="multipartite_rank"` (aliases: `"multipartiterank"`, `"multipartite"`, `"mpr"`). Set `multipartite_alpha` and `multipartite_similarity_threshold` in the JSON config:

```python
import json
from rapid_textrank import extract_from_json

payload = {
    "tokens": tokens,  # Pre-tokenized (e.g., from spaCy)
    "variant": "multipartite_rank",
    "config": {
        "top_n": 10,
        "multipartite_alpha": 1.1,
        "multipartite_similarity_threshold": 0.26,
    },
}

result = json.loads(extract_from_json(json.dumps(payload)))
```

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `similarity_threshold` | float | 0.26 | Jaccard similarity threshold for grouping candidates into topic clusters. Higher values produce fewer, larger clusters. |
| `alpha` | float | 1.1 | Position boost strength. Controls how much first-occurring candidates within a cluster are favored. Set to 0 to disable the position boost entirely. |

## MultipartiteRank vs TopicRank

Both variants cluster candidates into topics, but they differ in graph construction and candidate handling:

- **TopicRank** collapses each topic into a single node and ranks topics as a whole, then picks the best representative from each. This gives strong diversity but loses fine-grained candidate distinctions.
- **MultipartiteRank** keeps every candidate as its own node but removes edges within the same topic. This preserves fine-grained candidate distinctions while still preventing intra-topic competition.

In practice, MultipartiteRank tends to produce more nuanced results because it can rank individual phrases rather than entire topic clusters. The `alpha` position boost further helps surface the most salient phrasing of each topic.

## When to Use MultipartiteRank

MultipartiteRank is a good choice for:

- **Long, multi-topic documents** where you want diverse keywords but also care about the specific phrasing of each keyword.
- **Documents with near-duplicate phrases** (e.g., "machine learning" and "learning machine") where you want the most natural variant to surface.
- **Scenarios where TopicRank's topic-level collapsing is too aggressive** -- MultipartiteRank offers a middle ground between per-candidate ranking (BaseTextRank) and per-topic ranking (TopicRank).

## Reference

- [Unsupervised Keyphrase Extraction with Multipartite Graphs](https://aclanthology.org/N18-2105/) (Boudin, 2018)
