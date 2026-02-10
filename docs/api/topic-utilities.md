# Topic Utilities

rapid_textrank includes a helper function for computing per-lemma topic weights from a trained gensim LDA model. These weights can be passed directly to `TopicalPageRank` for topic-model-guided keyword extraction.

## Installation

The topic utilities require gensim:

```bash
pip install rapid_textrank[topic]
```

## topic_weights_from_lda

Computes per-lemma importance weights from a trained LDA model and a single document's bag-of-words representation.

### Signature

```python
topic_weights_from_lda(
    lda_model,
    corpus_entry: list[tuple[int, int | float]],
    dictionary,
    top_n_words: int = 50,
    aggregation: str = "max",
) -> dict[str, float]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `lda_model` | `gensim.models.LdaModel` | *(required)* | A trained gensim LDA model (or `LdaMulticore`). |
| `corpus_entry` | `list[tuple[int, int]]` | *(required)* | Bag-of-words for a single document, as returned by `dictionary.doc2bow(tokens)`. |
| `dictionary` | `gensim.corpora.Dictionary` | *(required)* | The gensim `Dictionary` mapping token IDs to words. |
| `top_n_words` | `int` | `50` | Number of top words to retrieve per topic. |
| `aggregation` | `str` | `"max"` | How to aggregate a word's weight across multiple topics. `"max"` keeps the highest weight; `"mean"` averages. |

### Returns

A `dict[str, float]` mapping lemma strings to importance weights, suitable for passing to `TopicalPageRank(topic_weights=...)`.

### How It Works

For each topic that the document belongs to, the function retrieves the top words and computes `P(topic|doc) * P(word|topic)` for every word. Scores are then aggregated across topics using the specified method.

## Full Example

```python
from gensim.corpora import Dictionary
from gensim.models import LdaModel
from rapid_textrank import TopicalPageRank, topic_weights_from_lda

# 1. Train (or load) an LDA model
corpus = [
    "transformers attention neural networks deep learning",
    "access control authentication encryption audit logging",
    "renewable energy solar wind grid storage batteries",
    "customer retention cohort analysis activation funnel",
    "privacy gdpr consent tracking cookies analytics",
]

texts = [doc.split() for doc in corpus]
dictionary = Dictionary(texts)
bow_corpus = [dictionary.doc2bow(t) for t in texts]
lda = LdaModel(bow_corpus, num_topics=3, id2word=dictionary, random_state=0)

# 2. Compute topic weights for a single document
doc_id = 4
raw_text = corpus[doc_id]
weights = topic_weights_from_lda(lda, bow_corpus[doc_id], dictionary)

# 3. Extract keywords using those weights
extractor = TopicalPageRank(
    topic_weights=weights,
    min_weight=0.01,
    top_n=12,
    language="en",
)

result = extractor.extract_keywords(raw_text)
for phrase in result.phrases[:10]:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

## Batch Pipeline Pattern

For processing many documents, keep a single `TopicalPageRank` instance and pass new weights per call:

```python
extractor = TopicalPageRank(top_n=10, language="en")

for doc_id in range(len(bow_corpus)):
    weights = topic_weights_from_lda(lda, bow_corpus[doc_id], dictionary)
    result = extractor.extract_keywords(
        corpus[doc_id],
        topic_weights=weights,
    )
    print(f"Doc {doc_id}: {[p.text for p in result.phrases[:5]]}")
```

## Aggregation Modes

| Mode | Behavior |
|------|----------|
| `"max"` (default) | For each word, keep the highest `P(topic|doc) * P(word|topic)` across all topics. Good when a word's importance is best captured by its strongest topic association. |
| `"mean"` | Average the weight across all topics the word appears in. Smooths out weights for words that appear across many topics. |

## Notes

- **Topic modeling is optional.** `TopicalPageRank` accepts any `dict[str, float]` as topic weights. You can supply TF-IDF weights, embedding similarities, domain relevance scores, or hand-picked values instead of LDA-derived weights.
- **Gensim is only imported on demand.** The `topic_weights_from_lda` function is lazily loaded to avoid pulling in gensim unless you actually call it.
