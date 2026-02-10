# Quick Start

## Extract Keywords in Five Lines

The fastest way to pull keywords from text is the `extract_keywords` convenience function:

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

Each returned `Phrase` object carries several attributes:

| Attribute | Description |
|-----------|-------------|
| `text`    | Surface form of the phrase (e.g., `"machine learning"`) |
| `lemma`   | Lemmatized form |
| `score`   | TextRank score |
| `count`   | Number of occurrences in the text |
| `rank`    | 1-indexed rank |

## Class-Based API

For more control -- choosing an algorithm variant, tuning configuration, or reusing an extractor across multiple documents -- use the class-based API:

```python
from rapid_textrank import BaseTextRank, TextRankConfig

config = TextRankConfig(
    top_n=10,
    language="en",
    min_phrase_length=2,
    max_phrase_length=4,
)

extractor = BaseTextRank(config=config)
result = extractor.extract_keywords(text)

for phrase in result.phrases:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

All seven algorithm variants (`BaseTextRank`, `PositionRank`, `BiasedTextRank`, `TopicRank`, `SingleRank`, `TopicalPageRank`, `MultipartiteRank`) follow the same pattern. See the [Extractor Classes](../api/extractor-classes.md) reference for the full list of constructors and parameters.

## JSON Interface

If you already tokenize with spaCy (or another NLP pipeline), you can pass pre-tokenized data directly via the JSON interface to avoid re-tokenizing in Rust:

```python
import json
from rapid_textrank import extract_from_json

payload = {
    "tokens": [
        {
            "text": "Machine",
            "lemma": "machine",
            "pos": "NOUN",
            "start": 0,
            "end": 7,
            "sentence_idx": 0,
            "token_idx": 0,
            "is_stopword": False,
        },
        # ... more tokens
    ],
    "variant": "textrank",
    "config": {"top_n": 10, "language": "en"},
}

result = json.loads(extract_from_json(json.dumps(payload)))
```

See the [JSON Interface](../api/json-interface.md) reference for full details on the payload schema and supported variants.

!!! tip "Interactive Notebook"
    Explore this topic in the [Quick Start Notebook](https://github.com/xang1234/rapid-textrank/blob/main/notebooks/01_quickstart.ipynb).
