# Release Notes

## v0.1.4

### Bug Fixes

- **Fixed missing `MultipartiteRank` Python export** -- the class was registered in the Rust PyO3 module but not re-exported from `python/rapid_textrank/__init__.py`, causing `ImportError: cannot import name 'MultipartiteRank'` when installing from PyPI.

---

## v0.1.3

### New Algorithm Variants

#### SingleRank
- New `SingleRank` variant with **weighted edges** based on co-occurrence frequency and **cross-sentence windowing** -- words in adjacent sentences can co-occur within the same sliding window, unlike classic TextRank which treats each sentence independently.
- Full Python bindings: native class, JSON dispatch, and spaCy pipeline component.
- Unit tests, integration tests, and documentation.

#### TopicalPageRank
- New `TopicalPageRank` variant that biases PageRank toward user-supplied **topic weights**, enabling domain-focused keyword extraction via a personalization vector.
- Includes `PersonalizedPageRank` engine in `src/pagerank/personalized.rs` with automatic normalization of the personalization vector.
- 7 unit tests and an integration pipeline test.
- Python `topic_weights_from_lda` helper (`python/rapid_textrank/topic_utils.py`) for deriving topic weights from a Gensim LDA model, with an optional Jaccard pre-filter heuristic.
- Full Python bindings and README examples.

#### MultipartiteRank
- New `MultipartiteRank` variant that performs **Hierarchical Agglomerative Clustering (HAC)** on candidate keyphrases using Jaccard distance, then applies a multipartite graph ranking.
- Shared `src/clustering.rs` module extracted from the former TopicRank-specific clustering code, now reused by both TopicRank and MultipartiteRank.
- Disjoint-set fast-path optimization for Jaccard distance computation.
- Full Python bindings, README documentation, and notebook examples.

### Core Engine Improvements

- **Cross-sentence windowing** added to `GraphBuilder` -- configurable window that spans sentence boundaries, improving recall for long documents.
- **Shared clustering module** (`src/clustering.rs`) -- HAC with Jaccard distance, reusable across variants.
- **TopicRank edge weighting** aligned with the PKE reference implementation for correctness.

### Bug Fixes

- **Fixed `use_pos_in_nodes` serde default** -- changed from `false` to `true`, fixing 2 POS-filtering tests (`test_json_include_pos_filtering`, `test_json_include_pos_multiple_tags`).
- **Fixed `cargo fmt` violations** that were failing CI checks across multiple files.

### Documentation & Examples

- README updated with usage examples for SingleRank, TopicalPageRank, and MultipartiteRank.
- Notebooks `02_algorithm_variants` and `04_benchmarks` updated with examples for all new variants.
- `CLAUDE.md` and `AGENTS.md` updated with PyO3 + Python 3.14 build caveats.
- `topic_weights_from_lda` usage example added to README.

### Benchmarks

- Added SingleRank, TopicalPageRank, and MultipartiteRank to the Criterion benchmark suite.

### Stats

- **20 commits** since v0.1.2
- **27 files changed**, **3,409 insertions**, **271 deletions**

---

## v0.1.2

### Highlights
- Added TopicRank support via JSON interface (`variant="topic_rank"`) with spaCy-token examples.
- TopicRank behavior aligned more closely with pytextrank.
- Docs and notebooks updated to include TopicRank usage and comparisons.

### Details
- New JSON config fields: `topic_similarity_threshold`, `topic_edge_weight`, `focus_terms`, `bias_weight`.
- README updated with TopicRank section + citation.
- Benchmarks notebook now compares rapid_textrank TopicRank vs pytextrank TopicRank using spaCy tokens.

---

## v0.1.1

### Highlights
- **Closer alignment with pytextrank defaults**: window size now 3, POS defaults include verbs, and scrubbed-text grouping available by default.
- **Improved phrase quality**: stopword-aware chunking reduces noisy phrases.
- **Config parity across APIs**: new options surfaced in Python, JSON, and spaCy component interfaces.

### New
- `use_pos_in_nodes` to treat nodes as `lemma|POS`.
- `phrase_grouping` with `lemma` or `scrubbed_text`.
- `get_stopwords(language)` helper to inspect built-in stopwords.
- JSON config now supports `language`, `use_pos_in_nodes`, `phrase_grouping`, and additional stopwords (extends built-ins).
- spaCy component supports `include_pos`, `use_pos_in_nodes`, `phrase_grouping`, `language`, and `stopwords`.

### Changed Defaults
- `window_size`: **4 to 3**
- `include_pos`: **NOUN + ADJ + PROPN to NOUN + ADJ + PROPN + VERB**
- `use_pos_in_nodes`: **false to true**
- `phrase_grouping`: **lemma to scrubbed_text**

### Notebook Updates
- Benchmarks now compare rapid_textrank vs pytextrank **with and without spaCy tokens**.
- Config blocks simplified to use new defaults.
- Stopword list printing added to algorithm explanation notebook.

### Bug Fixes
- Stopwords now act as **chunk boundaries**, preventing phrases like "of NLP is to".
- Heuristic POS tagging recognizes common function words to reduce false content tokens.

### Compatibility Notes
If you relied on old defaults, set them explicitly to preserve behavior:
- `window_size=4`
- `include_pos=["NOUN","ADJ","PROPN"]`
- `use_pos_in_nodes=False`
- `phrase_grouping="lemma"`

---

## v0.1.0

**Release Date:** February 5, 2026

This is the initial public release of `rapid_textrank`, a high-performance TextRank implementation in Rust with Python bindings.

### Highlights

- **10-100x faster** than pure Python TextRank implementations (depending on document size)
- **Three algorithm variants**: BaseTextRank, PositionRank, and BiasedTextRank
- **18 languages** supported for stopword filtering
- **Dual API**: Native Python classes + JSON interface for batch processing
- **spaCy integration**: Drop-in pipeline component

### Features

#### Algorithm Variants

| Variant | Use Case |
|---------|----------|
| `BaseTextRank` | General keyword extraction |
| `PositionRank` | Documents where key terms appear early (papers, news) |
| `BiasedTextRank` | Topic-focused extraction with customizable focus terms |

#### Performance Optimizations

- CSR (Compressed Sparse Row) graph format for cache-friendly PageRank iteration
- String interning via `StringPool` reducing memory 10-100x for typical documents
- Parallel graph construction with Rayon
- Link-time optimization (LTO) with single codegen unit
- FxHash for fast internal hash maps

### Supported Platforms

- Python 3.9, 3.10, 3.11, 3.12
- Linux (manylinux)
- macOS (x86_64, arm64)
- Windows (x86_64)

### Supported Languages

`en`, `de`, `fr`, `es`, `it`, `pt`, `nl`, `ru`, `sv`, `no`, `da`, `fi`, `hu`, `tr`, `pl`, `ar`, `zh`, `ja`
