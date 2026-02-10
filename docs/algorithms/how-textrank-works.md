# How TextRank Works

TextRank is a graph-based ranking algorithm for keyword extraction, inspired by Google's PageRank.

## The Algorithm

1. **Build a co-occurrence graph** -- Words become nodes. An edge connects two words if they appear within a sliding window (default: 4 words).

2. **Run PageRank** -- The algorithm iteratively distributes "importance" through the graph. Words connected to many important words become important themselves.

3. **Extract phrases** -- High-scoring words are grouped into noun chunks (POS-filtered) to form key phrases. Scores are aggregated (sum, mean, or max).

```
Text: "Machine learning enables systems to learn from data"

Co-occurrence graph (window=2):
    machine <-> learning <-> enables <-> systems <-> learn <-> data
                              |
                            PageRank
                              |
    Scores: machine(0.23) learning(0.31) enables(0.12) ...
                              |
                        Phrase extraction
                              |
    "machine learning" (0.54), "systems" (0.18), ...
```

Each [algorithm variant](index.md) modifies one or more of these steps:

- **PositionRank** biases PageRank teleportation toward words that appear early.
- **BiasedTextRank** biases teleportation toward user-specified focus terms.
- **SingleRank** weights edges by co-occurrence count and windows across sentence boundaries.
- **TopicalPageRank** biases teleportation using per-word topic weights (e.g., from LDA).
- **TopicRank** clusters candidates into topics, builds a topic-level graph, then picks representatives.
- **MultipartiteRank** keeps individual candidates but removes intra-topic edges and boosts first-occurring variants.

## Further Reading

- [TextRank: Bringing Order into Texts](https://aclanthology.org/W04-3252/) (Mihalcea & Tarau, 2004)
- [PositionRank: An Unsupervised Approach to Keyphrase Extraction](https://aclanthology.org/P17-1102/) (Florescu & Caragea, 2017)
- [BiasedTextRank: Unsupervised Graph-Based Content Extraction](https://aclanthology.org/2020.coling-main.144/) (Kazemi et al., 2020)
- [TopicRank: Graph-Based Topic Ranking for Keyphrase Extraction](https://aclanthology.org/I13-1062/) (Bougouin et al., 2013)
- [SingleRank: Single Document Keyphrase Extraction Using Neighborhood Knowledge](https://ojs.aaai.org/index.php/AAAI/article/view/7798) (Wan & Xiao, 2008)
- [Topical Word Importance for Fast Keyphrase Extraction](https://aclanthology.org/W15-3605/) (Sterckx et al., 2015)
- [MultipartiteRank: Unsupervised Keyphrase Extraction with Multipartite Graphs](https://aclanthology.org/N18-2105/) (Boudin, 2018)

!!! tip "Interactive Notebook"
    See the algorithm explained step-by-step in the [Algorithm Explanation Notebook](https://github.com/xang1234/rapid-textrank/blob/main/notebooks/03_explain_algorithm.ipynb).
