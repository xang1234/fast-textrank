# TopicRank

TopicRank (Bougouin et al., 2013) clusters similar candidate phrases into topics, builds a graph over those topics, ranks them with PageRank, and then selects the best representative phrase from each top-ranked topic. This approach promotes diversity -- the final keyword list covers distinct themes rather than repeating near-synonyms.

!!! warning "JSON Interface Only"
    TopicRank does not have a native Python class. Use the JSON interface with `variant="topic_rank"` and pre-tokenized input (e.g., from spaCy).

## How It Works

1. **Candidate extraction** -- Candidate phrases are identified using POS-filtered noun chunks (same as other variants).
2. **Topic clustering** -- Candidates are grouped into topics based on string similarity (Jaccard over word sets). The `topic_similarity_threshold` parameter controls how aggressively candidates are merged.
3. **Topic graph** -- A graph is built where each node is a topic (cluster). Edges are weighted by the co-occurrence of candidates across different topics.
4. **PageRank on topics** -- Standard PageRank ranks the topic nodes.
5. **Representative selection** -- From each top-ranked topic, the best candidate phrase is selected as the representative.

## Usage

TopicRank requires pre-tokenized input, which makes it a natural fit for spaCy-based pipelines.

### Install spaCy

```bash
pip install rapid_textrank[spacy]
python -m spacy download en_core_web_sm
```

### Full example

```python
import json
import spacy
from rapid_textrank import extract_from_json

nlp = spacy.load("en_core_web_sm")
doc = nlp(text)

tokens = []
for sent_idx, sent in enumerate(doc.sents):
    for token in sent:
        tokens.append({
            "text": token.text,
            "lemma": token.lemma_,
            "pos": token.pos_,
            "start": token.idx,
            "end": token.idx + len(token.text),
            "sentence_idx": sent_idx,
            "token_idx": token.i,
            "is_stopword": token.is_stop,
        })

payload = {
    "tokens": tokens,
    "variant": "topic_rank",
    "config": {
        "top_n": 10,
        "language": "en",
        "topic_similarity_threshold": 0.25,
        "topic_edge_weight": 1.0,
    },
}

result = json.loads(extract_from_json(json.dumps(payload)))
for phrase in result["phrases"][:10]:
    print(phrase["text"], phrase["score"])
```

## Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `topic_similarity_threshold` | float | 0.25 | Jaccard similarity threshold for grouping candidates into topics. Higher values produce fewer, larger topics (more aggressive clustering). |
| `topic_edge_weight` | float | 1.0 | Base weight for edges between topic nodes in the topic graph. |

These fields are set inside the `config` object of the JSON payload, alongside standard fields like `top_n` and `language`.

## When to Use TopicRank

TopicRank is designed for documents that span multiple themes, where vanilla TextRank tends to over-represent the dominant topic:

- **Quarterly reports** covering product, finance, security, and compliance.
- **Long-form articles** with multiple sections on different subtopics.
- **Meeting notes** spanning several agenda items.

If you want topic-based diversity but also need fine-grained candidate distinctions (rather than collapsing each topic to a single representative), consider [MultipartiteRank](multipartite-rank.md).

## Reference

- [TopicRank: Graph-Based Topic Ranking for Keyphrase Extraction](https://aclanthology.org/I13-1062/) (Bougouin et al., 2013)
