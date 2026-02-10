# Recipes

These recipes are grab-and-go starting points for common real-world tasks. Each one is a self-contained example you can copy, paste, and adapt.

**Notes:**

- Outputs will vary by input text (and by tokenization if you use spaCy vs the built-in tokenizer).
- If you want to tune phrase shapes (2--4 word phrases, noun-only phrases, grouping behavior, etc.), see the [TextRankConfig](../api/textrank-config.md) reference.

---

## 1. SEO / Blog Keywording (BaseTextRank + Phrase Grouping)

**Goal:** extract "SEO-ready" keyphrases (usually 2--4 words) from a blog post draft or landing page copy.

This recipe emphasizes:

- Multi-word phrases (to avoid a list of generic single tokens).
- Noun/adjective/proper-noun phrases (to filter out noisy verbs).
- Scrubbed-text grouping (so phrases look like what you would actually put in a CMS or brief).
- Optional lemma-deduping (to avoid near-duplicates in your final list).

```python
from rapid_textrank import BaseTextRank, TextRankConfig

text = """
Privacy-first analytics is becoming the default choice for SaaS teams who
want accurate conversion tracking without relying on third-party cookies.
In this guide, we'll compare cookieless tracking, first-party event capture,
self-hosted deployment, and GDPR-friendly consent flows.

You'll learn how to:
- pick a privacy-first analytics tool for product and marketing,
- define an event taxonomy (signup, activation, retention),
- build dashboards for funnel conversion and cohort retention,
- ship tracking with minimal performance impact.
"""

config = TextRankConfig(
    top_n=25,
    language="en",

    # SEO keyword lists generally benefit from longer phrases.
    min_phrase_length=2,
    max_phrase_length=4,

    # Most SEO phrases are noun-y. Dropping VERB helps reduce "learn how" style noise.
    include_pos=["NOUN", "ADJ", "PROPN"],

    # Group phrases by scrubbed surface form so outputs look "human" (vs pure lemmas).
    phrase_grouping="scrubbed_text",

    # Add your own "boilerplate" stopwords (brand name, CTA words, years, etc.).
    stopwords=[
        "guide", "tutorial", "tips", "step", "steps",
        "learn", "using", "use", "how",
        "2026",  # years often sneak into keyword lists
    ],
)

extractor = BaseTextRank(config=config)
result = extractor.extract_keywords(text)

# Optional: dedupe by lemma so you don't end up with near-duplicates.
seen = set()
keywords = []
for phrase in result.phrases:
    key = phrase.lemma
    if key not in seen:
        seen.add(key)
        keywords.append(phrase.text)

print(keywords[:15])
```

Example output (will vary by input text):

```
[
  'privacy-first analytics',
  'cookieless tracking',
  'first-party event capture',
  'self-hosted deployment',
  'gdpr-friendly consent flows',
  'event taxonomy',
  'funnel conversion',
  'cohort retention',
  'conversion tracking',
  'performance impact'
]
```

**Practical tweaks for SEO workflows:**

- Run on: H1 + first 2--3 paragraphs + headings. Full posts can dilute intent.
- Add stopwords for your brand/product name if it dominates results.
- If you need canonical forms for spreadsheets: use `phrase.lemma` instead of `phrase.text`.

---

## 2. Academic Abstract Extraction (PositionRank)

**Goal:** extract keyphrases from paper abstracts (or news articles) where important terms are typically introduced early. PositionRank biases the random walk toward early-occurring candidates, which often matches academic writing style.

**Tip:** include the paper title and abstract together in one string -- titles are short but highly informative.

```python
from rapid_textrank import PositionRank

text = """
Title: Efficient Graph-Based Keyphrase Extraction with Sparse Updates

Abstract: Keyphrase extraction is commonly used to index scientific documents,
support literature discovery, and improve retrieval. We propose a sparse-update
graph ranking method that reduces computational cost while preserving phrase quality.
Experiments on benchmark datasets show improved F1 under strict matching.
"""

extractor = PositionRank(top_n=12, language="en")
result = extractor.extract_keywords(text)

# Print the top phrases (scores shown for debugging; sort order is by score).
for phrase in result.phrases[:10]:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

**When PositionRank usually helps:**

- Short, structured documents (title + abstract).
- News leads (topic stated upfront).
- Executive summaries (key themes early).

**If you want fewer generic results:**

Increase `top_n` and then keep only 2--4 word phrases in post-processing:

```python
filtered = [p.text for p in result.phrases if 2 <= len(p.text.split()) <= 4]
```

---

## 3. Security / Privacy-Focused Extraction (BiasedTextRank)

**Goal:** pull out security- or privacy-relevant phrases from a mixed document (policies, architecture docs, incident reports, DPIAs, vendor questionnaires). BiasedTextRank lets you "steer" extraction toward your focus terms while still discovering related phrases.

```python
from rapid_textrank import BiasedTextRank

text = """
We encrypt data at rest using AES-256 and enforce TLS 1.2+ for data in transit.
Access to production is gated by MFA and short-lived credentials. Audit logs are
retained for 180 days and monitored for anomalous access patterns. Personal data
processing is limited to the declared purpose, and retention follows a documented
schedule. We support DSAR workflows and apply data minimization by default.
"""

extractor = BiasedTextRank(
    # Keep focus terms simple (single tokens). Think: what you'd search for in the doc.
    focus_terms=[
        "privacy", "personal", "pii", "encrypt", "encryption",
        "tls", "mfa", "audit", "retention", "dsar", "minimization"
    ],
    bias_weight=8.0,   # 5-10 is a reasonable starting band
    top_n=15,
    language="en",
)

result = extractor.extract_keywords(text)
for phrase in result.phrases[:10]:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

**Two useful patterns:**

- **Query-driven extraction:** set `focus_terms` from user input (e.g., "SOC2 audit logging").
- **Per-call focus override** (same extractor, different focus sets):

```python
result = extractor.extract_keywords(text, focus_terms=["privacy", "retention", "dsar"])
```

---

## 4. Topic-Model Guided Keywords from LDA (TopicalPageRank)

**Goal:** if you already run LDA (or any topic model) on a corpus, use the topic distribution to bias keyword extraction automatically. TopicalPageRank uses a personalization vector where words with higher topic weights get more teleport probability.

This recipe uses the built-in `topic_weights_from_lda` helper for gensim LDA.

```bash
pip install rapid_textrank[topic]   # installs gensim
```

```python
from gensim.corpora import Dictionary
from gensim.models import LdaModel
from rapid_textrank import TopicalPageRank, topic_weights_from_lda

# Tiny example corpus (replace with your real corpus)
corpus = [
    "transformers attention neural networks deep learning",
    "access control authentication encryption audit logging",
    "renewable energy solar wind grid storage batteries",
    "customer retention cohort analysis activation funnel",
    "privacy gdpr consent tracking cookies analytics",
]

texts = [doc.lower().split() for doc in corpus]
dictionary = Dictionary(texts)
bow_corpus = [dictionary.doc2bow(t) for t in texts]

lda = LdaModel(bow_corpus, num_topics=3, id2word=dictionary, random_state=0)

# Pick a document and compute topic weights for it
doc_id = 4
raw_text = corpus[doc_id]
weights = topic_weights_from_lda(lda, bow_corpus[doc_id], dictionary)

extractor = TopicalPageRank(
    topic_weights=weights,
    min_weight=0.01,   # floor for words missing from the topic weights
    top_n=12,
    language="en",
)

result = extractor.extract_keywords(raw_text)
for phrase in result.phrases[:10]:
    print(f"{phrase.text}: {phrase.score:.4f}")
```

**Operational tips:**

- For batch pipelines, keep one `TopicalPageRank` instance and pass new weights per call:

```python
result = extractor.extract_keywords(other_text, topic_weights=other_weights)
```

- `topic_weights_from_lda` supports `aggregation` (`"max"` or `"mean"`) and `top_n_words` to tune how per-topic weights get combined.

---

## 5. Multi-Topic Documents (TopicRank / MultipartiteRank)

**Goal:** extract diverse phrases from long documents that cover multiple themes (e.g., quarterly reports, RFCs, incident postmortems). Vanilla TextRank can over-focus on the dominant cluster.

Two strong options:

- **TopicRank** clusters candidates into topics, ranks topics, then selects representatives (diversity-first).
- **MultipartiteRank** keeps candidates but removes intra-topic edges and can boost early variants (more nuanced).

### TopicRank (via JSON interface -- great if you already tokenize with spaCy)

```bash
pip install rapid_textrank[spacy]
python -m spacy download en_core_web_sm
```

```python
import json
import spacy
from rapid_textrank import extract_from_json

text = """
Product: We shipped a new recommendation system and improved onboarding conversion.
Security: We rolled out MFA for all admins and expanded audit logging for production access.
Finance: Revenue grew 18% QoQ and churn decreased after pricing changes.
Compliance: We updated our retention policy and improved DSAR response automation.
"""

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
        "top_n": 15,
        "language": "en",

        # Higher threshold => fewer, larger topics (more aggressive clustering).
        "topic_similarity_threshold": 0.25,
        "topic_edge_weight": 1.0,
    },
}

result = json.loads(extract_from_json(json.dumps(payload)))
print([p["text"] for p in result["phrases"][:12]])
```

### MultipartiteRank (native class -- no spaCy needed)

```python
from rapid_textrank import MultipartiteRank

extractor = MultipartiteRank(
    similarity_threshold=0.26,
    alpha=1.1,      # 0 disables the position boost
    top_n=15,
    language="en",
)
result = extractor.extract_keywords(text)
print([p.text for p in result.phrases[:12]])
```

**If you want even more diversity:**

- Split the document by headings or sections and run extraction per section, then merge and deduplicate.
