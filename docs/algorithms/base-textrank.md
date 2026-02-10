# BaseTextRank

BaseTextRank is the standard TextRank implementation -- a direct adaptation of the algorithm described by Mihalcea and Tarau (2004). It is the default variant used by the convenience function `extract_keywords()` and a solid general-purpose choice for keyword extraction.

## How It Works

BaseTextRank follows the classic three-step pipeline:

1. Build an unweighted co-occurrence graph from a sliding window over the text.
2. Run PageRank with uniform teleportation to score each word node.
3. Group high-scoring words into phrases using POS-filtered noun chunks.

For a detailed walkthrough of these steps, see [How TextRank Works](how-textrank-works.md).

## Usage

### Convenience function

The simplest way to use BaseTextRank is through `extract_keywords()`, which creates a BaseTextRank extractor under the hood:

```python
from rapid_textrank import extract_keywords

keywords = extract_keywords(text, top_n=10, language="en")
for phrase in keywords:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

### Class-based API

For more control, instantiate `BaseTextRank` directly:

```python
from rapid_textrank import BaseTextRank

extractor = BaseTextRank(top_n=10, language="en")
result = extractor.extract_keywords(text)

for phrase in result.phrases:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

### With TextRankConfig

For full configuration (window size, POS filtering, phrase length, score aggregation, and more):

```python
from rapid_textrank import BaseTextRank, TextRankConfig

config = TextRankConfig(
    top_n=10,
    language="en",
    window_size=3,
    min_phrase_length=2,
    max_phrase_length=4,
    include_pos=["NOUN", "ADJ", "PROPN"],
    score_aggregation="sum",
)

extractor = BaseTextRank(config=config)
result = extractor.extract_keywords(text)
```

## When to Use BaseTextRank

BaseTextRank is the right starting point when:

- You have no prior knowledge about the document's topics or structure.
- You want a simple, reliable baseline with no extra configuration.
- Your documents are of moderate length and cover a single dominant topic.

If you need to steer results toward specific terms, consider [BiasedTextRank](biased-textrank.md). For documents where key terms appear early, try [PositionRank](position-rank.md). For longer or multi-topic documents, see [SingleRank](single-rank.md), [TopicRank](topic-rank.md), or [MultipartiteRank](multipartite-rank.md).

## Reference

- [TextRank: Bringing Order into Texts](https://aclanthology.org/W04-3252/) (Mihalcea & Tarau, 2004)
