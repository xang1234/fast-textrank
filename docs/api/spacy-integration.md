# spaCy Integration

rapid_textrank provides a spaCy pipeline component that uses the Rust core for keyword extraction while integrating seamlessly with spaCy's NLP pipeline. It can be used as a drop-in replacement for pytextrank with significantly better performance.

## Installation

```bash
pip install rapid_textrank[spacy]
python -m spacy download en_core_web_sm
```

## Basic Usage

```python
import spacy
import rapid_textrank.spacy_component  # registers the pipeline factory

nlp = spacy.load("en_core_web_sm")
nlp.add_pipe("rapid_textrank")

doc = nlp("Machine learning is a subset of artificial intelligence.")
for phrase in doc._.phrases[:5]:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

Importing `rapid_textrank.spacy_component` registers a spaCy pipeline factory named `"rapid_textrank"`. After `add_pipe`, every `nlp(text)` call will automatically run keyword extraction and store results on the `Doc`.

## Doc Extensions

The component registers two custom extensions on `spacy.tokens.Doc`:

| Extension | Type | Description |
|-----------|------|-------------|
| `doc._.phrases` | `list[Phrase]` | Extracted phrases, sorted by score descending. Each `Phrase` has `text`, `lemma`, `score`, `count`, and `rank` attributes. |
| `doc._.textrank_result` | `RustTextRankResult` | Full result object with `phrases`, `converged`, and `iterations` attributes. |

## Configuration

The pipeline component accepts all configuration parameters at `add_pipe` time:

```python
nlp.add_pipe("rapid_textrank", config={
    "top_n": 15,
    "language": "en",
    "include_pos": ["NOUN", "ADJ", "PROPN"],
    "use_pos_in_nodes": True,
    "phrase_grouping": "scrubbed_text",
    "window_size": 4,
    "min_phrase_length": 2,
    "max_phrase_length": 4,
    "stopwords": ["example", "custom"],
    "variant": "textrank",
})
```

### Supported config parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `damping` | `float` | `0.85` | PageRank damping factor. |
| `max_iterations` | `int` | `100` | Maximum PageRank iterations. |
| `convergence_threshold` | `float` | `1e-6` | Convergence threshold. |
| `window_size` | `int` | `3` | Co-occurrence window size. |
| `top_n` | `int` | `10` | Number of phrases to return. |
| `min_phrase_length` | `int` | `1` | Minimum words in a phrase. |
| `max_phrase_length` | `int` | `4` | Maximum words in a phrase. |
| `score_aggregation` | `str` | `"sum"` | Score aggregation method. |
| `include_pos` | `list[str]` | `["ADJ","NOUN","PROPN","VERB"]` | POS tags to include. |
| `use_pos_in_nodes` | `bool` | `True` | Use lemma+POS as graph node keys. |
| `phrase_grouping` | `str` | `"scrubbed_text"` | Phrase grouping strategy. |
| `language` | `str` | `"en"` | Language for stopwords. |
| `stopwords` | `list[str]` | `None` | Additional stopwords. |
| `variant` | `str` | `"textrank"` | Algorithm variant string. |

The `variant` parameter accepts the same variant strings as the [JSON interface](json-interface.md) (e.g., `"position_rank"`, `"biased_textrank"`, etc.).

## How It Works

Under the hood, the spaCy component:

1. Iterates over spaCy tokens and converts them to the JSON token format (using `token.text`, `token.lemma_`, `token.pos_`, `token.idx`, `token.i`, `token.is_stop`).
2. Calls `extract_from_json()` with the token data and configuration.
3. Parses the JSON result and stores `Phrase` objects on `doc._.phrases`.

This means you get the benefit of spaCy's tokenization, lemmatization, and POS tagging combined with rapid_textrank's fast Rust-based ranking.

## Serialization

The component supports spaCy's `to_disk` / `from_disk` methods, so pipelines can be saved and loaded:

```python
nlp.to_disk("./my_pipeline")
nlp2 = spacy.load("./my_pipeline")
```
