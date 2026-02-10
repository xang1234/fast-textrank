# FAQ

Frequently asked questions about rapid_textrank, covering common gotchas and configuration choices.

---

**Q: Why is TopicRank JSON-only?**

A: TopicRank requires pre-tokenized input with POS tags and lemmas, which is best provided by a full NLP pipeline like spaCy. The JSON interface accepts this pre-tokenized data directly. The other variants include a built-in tokenizer that handles tokenization, POS tagging, and lemmatization internally, so they can work with raw text via the native Python classes.

---

**Q: Is the Python GIL held during extraction?**

A: Yes, the GIL is currently held during extraction. For CPU-bound batch workloads, use `extract_batch_from_json()` to process multiple documents in a single Rust call, which avoids repeated Python-to-Rust boundary crossings. For true parallelism, use Python's `multiprocessing` module to run extractions across multiple processes.

---

**Q: How do stopwords work?**

A: The `stopwords` parameter in `TextRankConfig` adds to the built-in stopword list for the specified language -- it does not replace it. Use `get_stopwords("en")` to see the built-in list for a given language. If you need to remove a built-in stopword, there is currently no mechanism for that; the parameter is additive only.

---

**Q: How are emojis handled?**

A: Emojis are ignored by the built-in tokenizer. They will not appear as keywords or affect the co-occurrence graph. If you need emoji-aware processing, tokenize with spaCy (or another NLP tool that handles emojis) and use the JSON interface to pass the pre-tokenized data to rapid_textrank.

---

**Q: How is this different from pytextrank?**

A: rapid_textrank is a Rust implementation with Python bindings, while pytextrank is pure Python built on spaCy. rapid_textrank is 10-100x faster and offers a standalone API (no spaCy required for most variants), but pytextrank integrates more deeply into spaCy pipelines. If you are already using spaCy for NER, dependency parsing, or other linguistic features, pytextrank fits naturally into that workflow. If speed is your priority or you do not need spaCy, rapid_textrank is the better choice.

---

**Q: How do I tune for short vs long documents?**

A: For short documents (tweets, titles): use a smaller `window_size` (2), and consider increasing `max_phrase_length` to capture the few meaningful phrases. For long documents: use `SingleRank` (cross-sentence windowing improves recall) or `MultipartiteRank` (topic clustering promotes diversity). Consider adjusting `top_n` relative to document length -- a 50-word tweet might only have 3-5 meaningful keyphrases, while a 5,000-word article might justify 20-30.

---

**Q: What does `use_pos_in_nodes` do?**

A: When `True` (the default), graph nodes are keyed as `"lemma|POS"` (e.g., `"learning|NOUN"` vs `"learning|VERB"`). This prevents different word senses from sharing a node, which improves precision when the same surface form is used as both a noun and a verb. Set to `False` to collapse all POS variants of the same lemma into one node, which can improve recall at the cost of conflating different senses.

---

**Q: What does `phrase_grouping` do?**

A: Controls how phrases are grouped for deduplication. `"scrubbed_text"` (the default) groups by lowercase surface form, so "Machine Learning" and "machine learning" merge into one entry. `"lemma"` groups by lemmatized form, merging "machine learning" and "machine learns" into one entry. Use `"lemma"` when you want canonical forms (e.g., for spreadsheets or databases); use `"scrubbed_text"` when you want output that reads naturally.
