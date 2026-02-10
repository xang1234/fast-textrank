# extract_keywords()

The convenience function for one-liner keyword extraction. It uses `BaseTextRank` internally and returns a flat list of `Phrase` objects.

## Signature

```python
extract_keywords(text: str, top_n: int = 10, language: str = "en") -> list[Phrase]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `text` | `str` | *(required)* | The input text to extract keywords from. |
| `top_n` | `int` | `10` | Number of top keywords to return. |
| `language` | `str` | `"en"` | Language code for stopword filtering (see [Supported Languages](supported-languages.md)). |

### Returns

A `list` of [`Phrase`](result-objects.md#phrase) objects, sorted by score in descending order.

## Example

```python
from rapid_textrank import extract_keywords

text = """
Machine learning is a subset of artificial intelligence that enables
systems to learn and improve from experience. Deep learning, a type of
machine learning, uses neural networks with many layers.
"""

keywords = extract_keywords(text, top_n=5, language="en")
for phrase in keywords:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

Output:

```
machine learning: 0.2341
deep learning: 0.1872
artificial intelligence: 0.1654
neural networks: 0.1432
systems: 0.0891
```

## When to Use

`extract_keywords()` is the fastest path to results. Use it when:

- You want a one-liner with sensible defaults.
- You do not need to configure the algorithm beyond `top_n` and `language`.
- You are processing a single document and do not need to reuse an extractor instance.

For more control over the algorithm (damping factor, window size, POS filtering, phrase grouping, etc.), use the [extractor classes](extractor-classes.md) with a [`TextRankConfig`](textrank-config.md).
