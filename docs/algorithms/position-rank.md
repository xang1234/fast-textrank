# PositionRank

PositionRank (Florescu & Caragea, 2017) extends TextRank by weighting words according to their position in the document -- earlier appearances score higher. This captures the common writing convention of introducing key terms early (in titles, abstracts, or opening paragraphs).

## How It Works

PositionRank modifies the PageRank teleportation vector so that words appearing earlier receive a larger share of the teleport probability. The co-occurrence graph and phrase extraction steps remain the same as [BaseTextRank](base-textrank.md).

## Usage

```python
from rapid_textrank import PositionRank

extractor = PositionRank(top_n=10)
result = extractor.extract_keywords("""
Quantum Computing Advances in 2024

Researchers have made significant breakthroughs in quantum error correction.
The quantum computing field continues to evolve rapidly...
""")

# "quantum computing" and "quantum" will rank higher due to early position
```

PositionRank is also available via the JSON interface with `variant="position_rank"`.

## When to Use PositionRank

PositionRank is most effective on short, structured documents where important terms are introduced early:

- **Academic abstracts** -- key contributions are typically stated in the first few sentences.
- **News articles** -- the "inverted pyramid" style puts the most important facts first.
- **Executive summaries** -- themes are laid out upfront before supporting detail.
- **Title + abstract combinations** -- concatenate the title and abstract into one string for best results, since the title carries strong positional signal.

For longer documents where key terms are distributed throughout, [SingleRank](single-rank.md) or [BaseTextRank](base-textrank.md) may perform better because PositionRank's early-position bias can underweight terms that appear only in later sections.

## Reference

- [PositionRank: An Unsupervised Approach to Keyphrase Extraction from Scholarly Documents](https://aclanthology.org/P17-1102/) (Florescu & Caragea, 2017)
