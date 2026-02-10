# SingleRank

SingleRank (Wan & Xiao, 2008) extends TextRank in two ways: edges are weighted by co-occurrence count (repeated neighbors get stronger connections), and the sliding window ignores sentence boundaries so that terms at the end of one sentence connect to terms at the start of the next.

## How It Works

In BaseTextRank, edges are binary (present or absent) and the window resets at sentence boundaries. SingleRank changes both behaviors:

1. **Weighted edges** -- If two words co-occur within the window multiple times, the edge weight accumulates. Frequently co-occurring pairs get a stronger signal.
2. **Cross-sentence windowing** -- The sliding window spans across sentence boundaries, so the last words of one sentence and the first words of the next can form edges. This captures co-occurrences that BaseTextRank misses.

PageRank then operates on this weighted graph, and phrase extraction proceeds as usual.

## Usage

```python
from rapid_textrank import SingleRank

extractor = SingleRank(top_n=10)
result = extractor.extract_keywords("""
Machine learning is a powerful tool. Deep learning is a subset of
machine learning. Neural networks power deep learning systems.
""")

# Cross-sentence co-occurrences strengthen "machine learning" edges
for phrase in result.phrases:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

SingleRank is also available via the JSON interface with `variant="single_rank"`.

## When to Use SingleRank over BaseTextRank

SingleRank works well on longer documents where important terms co-occur across sentence boundaries. The weighted edges amplify frequently co-occurring pairs, giving a clearer signal than the binary edges used by BaseTextRank.

Concrete scenarios where SingleRank tends to outperform BaseTextRank:

- **Long technical documents** -- co-occurrences accumulate over many sentences, and the weighted edges surface the dominant term pairs.
- **Documents with short sentences** -- when sentences are terse (e.g., bullet-point summaries), cross-sentence windowing recovers co-occurrence signal that would otherwise be lost.
- **Multi-paragraph prose** -- important terms often bridge paragraph and sentence boundaries; SingleRank captures these connections.

For short, structured documents (titles, abstracts), [PositionRank](position-rank.md) is usually a better choice. For multi-topic documents, consider [TopicRank](topic-rank.md) or [MultipartiteRank](multipartite-rank.md).

## Reference

- [Single Document Keyphrase Extraction Using Neighborhood Knowledge](https://ojs.aaai.org/index.php/AAAI/article/view/7798) (Wan & Xiao, 2008)
