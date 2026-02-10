# TopicalPageRank

TopicalPageRank (Sterckx et al., 2015) extends SingleRank by biasing the random walk toward topically important words. Instead of uniform teleportation, PageRank uses a personalization vector derived from per-word topic weights.

Users supply pre-computed topic weights as a `{lemma: weight}` dictionary. These typically come from a topic model (e.g., LDA via gensim or sklearn), but any source of word importance scores works -- TF-IDF weights, embedding similarities, domain relevance scores, or hand-picked values. Words absent from the dictionary receive a configurable minimum weight (`min_weight`, default 0.0).

## Usage

### Native Python class

```python
from rapid_textrank import TopicalPageRank

# Topic weights from an external topic model or manual assignment
topic_weights = {
    "neural": 0.9,
    "network": 0.8,
    "learning": 0.7,
    "deep": 0.6,
}

extractor = TopicalPageRank(
    topic_weights=topic_weights,
    min_weight=0.01,  # Floor for out-of-vocabulary words
    top_n=10
)

result = extractor.extract_keywords("""
Deep learning is a subset of machine learning that uses artificial neural
networks. Neural networks with many layers can learn complex patterns.
Convolutional neural networks excel at image recognition tasks.
""")

for phrase in result.phrases:
    print(f"{phrase.text}: {phrase.score:.4f}")

# Update topic weights for a different document/topic
result = extractor.extract_keywords(
    "Machine learning enables data-driven decisions...",
    topic_weights={"machine": 0.9, "data": 0.8}
)
```

### JSON interface

TopicalPageRank is also available via the JSON interface with `variant="topical_pagerank"` (aliases: `"tpr"`, `"single_tpr"`). Set `topic_weights` and optionally `topic_min_weight` in the JSON config:

```python
import json
from rapid_textrank import extract_from_json

payload = {
    "tokens": tokens,  # Pre-tokenized (e.g., from spaCy)
    "variant": "topical_pagerank",
    "config": {
        "top_n": 10,
        "topic_weights": {"neural": 0.9, "network": 0.8, "learning": 0.7},
        "topic_min_weight": 0.01,
    },
}

result = json.loads(extract_from_json(json.dumps(payload)))
```

## TopicalPageRank vs BiasedTextRank

Both variants bias extraction toward specific terms, but they differ in how the bias is specified:

- **BiasedTextRank** takes a list of focus terms and a single `bias_weight`. It is manual and direct -- good when you know exactly which terms matter.
- **TopicalPageRank** takes per-word weights, typically from a topic model. It is data-driven -- good when you want the topic distribution to guide extraction automatically.

Use BiasedTextRank when you have a short, hand-curated focus vocabulary. Use TopicalPageRank when you have a numeric importance score for each word (e.g., from LDA, TF-IDF, or embedding similarity).

## Topic Modeling is Optional

Despite the name, TopicalPageRank does not require a topic model. You can supply any word-importance dictionary:

- **TF-IDF weights** from a corpus-level vocabulary.
- **Embedding similarities** between candidate words and a target concept.
- **Domain relevance scores** from a domain-specific lexicon.
- **Hand-picked values** when you know the relative importance of key terms.

For details on computing topic weights from a gensim LDA model using the built-in `topic_weights_from_lda` helper, see [Topic Utilities](../api/topic-utilities.md).

## Reference

- [Topical Word Importance for Fast Keyphrase Extraction](https://aclanthology.org/W15-3605/) (Sterckx et al., 2015)
