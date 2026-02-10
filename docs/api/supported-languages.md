# Supported Languages

rapid_textrank includes built-in stopword lists for 18 languages. These are used for stopword filtering in all APIs: the convenience function, extractor classes, the JSON interface, and the spaCy component.

## Language Codes

| Code | Language | Code | Language | Code | Language |
|------|----------|------|----------|------|----------|
| `en` | English | `de` | German | `fr` | French |
| `es` | Spanish | `it` | Italian | `pt` | Portuguese |
| `nl` | Dutch | `ru` | Russian | `sv` | Swedish |
| `no` | Norwegian | `da` | Danish | `fi` | Finnish |
| `hu` | Hungarian | `tr` | Turkish | `pl` | Polish |
| `ar` | Arabic | `zh` | Chinese | `ja` | Japanese |

## Usage

Pass the language code to the `language` parameter in any API:

```python
from rapid_textrank import extract_keywords

# English (default)
phrases = extract_keywords(text, language="en")

# German
phrases = extract_keywords(german_text, language="de")

# Chinese
phrases = extract_keywords(chinese_text, language="zh")
```

With extractor classes:

```python
from rapid_textrank import BaseTextRank

extractor = BaseTextRank(top_n=10, language="fr")
result = extractor.extract_keywords(french_text)
```

With `TextRankConfig`:

```python
from rapid_textrank import TextRankConfig, BaseTextRank

config = TextRankConfig(language="ja")
extractor = BaseTextRank(config=config)
```

In the JSON interface:

```json
{
    "tokens": [ ... ],
    "config": {
        "language": "es"
    }
}
```

## Inspecting Stopwords

You can retrieve the built-in stopword list for any supported language:

```python
import rapid_textrank as rt

stopwords = rt.get_stopwords("en")
print(f"English stopwords: {len(stopwords)} words")
print(stopwords[:10])

stopwords_de = rt.get_stopwords("de")
print(f"German stopwords: {len(stopwords_de)} words")
```

## Extending Stopwords

The built-in lists can be extended with domain-specific terms using the `stopwords` parameter. These additional words are merged with the built-in list, not used as a replacement.

```python
from rapid_textrank import TextRankConfig, BaseTextRank

config = TextRankConfig(
    language="en",
    stopwords=["data", "system", "model"],  # added to built-in English stopwords
)

extractor = BaseTextRank(config=config)
```

In the JSON interface, the same applies via `config.stopwords`:

```json
{
    "tokens": [ ... ],
    "config": {
        "language": "en",
        "stopwords": ["data", "system", "model"]
    }
}
```
