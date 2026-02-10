# Installation

## From PyPI

The simplest way to install rapid_textrank:

```bash
pip install rapid_textrank
```

The import name is `rapid_textrank`.

## Optional Extras

### spaCy support

If you want to use the JSON interface with spaCy-tokenized input, or the spaCy pipeline component:

```bash
pip install rapid_textrank[spacy]
```

You will also need a spaCy model:

```bash
python -m spacy download en_core_web_sm
```

### Topic model support

If you plan to use `TopicalPageRank` with `topic_weights_from_lda` (gensim LDA integration):

```bash
pip install rapid_textrank[topic]
```

This installs [gensim](https://radimrehurek.com/gensim/) as an extra dependency.

## From Source

**Requirements:** Rust 1.70+, Python 3.9+

```bash
git clone https://github.com/xang1234/rapid-textrank
cd rapid_textrank
pip install maturin
maturin develop --release
```

This compiles the Rust core and installs the package into your current Python environment.

## Verifying the Installation

```python
import rapid_textrank
print(rapid_textrank.extract_keywords("Hello world", top_n=1))
```

If the import succeeds and returns a result list, the installation is working correctly.
