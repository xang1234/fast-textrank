# Citation

If you use rapid_textrank in research, please cite the relevant papers for the algorithm variants you use.

## TextRank

[TextRank: Bringing Order into Texts](https://aclanthology.org/W04-3252/) (Mihalcea & Tarau, 2004)

The foundational graph-based ranking algorithm for keyword extraction, inspired by PageRank. Used by `BaseTextRank`.

```bibtex
@inproceedings{mihalcea-tarau-2004-textrank,
    title = "{T}ext{R}ank: Bringing Order into Text",
    author = "Mihalcea, Rada and Tarau, Paul",
    booktitle = "Proceedings of EMNLP 2004",
    year = "2004",
    publisher = "Association for Computational Linguistics",
}
```

## PositionRank

[PositionRank: An Unsupervised Approach to Keyphrase Extraction from Scholarly Documents](https://aclanthology.org/P17-1102/) (Florescu & Caragea, 2017)

Extends TextRank by biasing the random walk toward words that appear earlier in the document. Used by `PositionRank`.

```bibtex
@inproceedings{florescu-caragea-2017-positionrank,
    title = "{P}osition{R}ank: An Unsupervised Approach to Keyphrase Extraction from Scholarly Documents",
    author = "Florescu, Corina and Caragea, Cornelia",
    booktitle = "Proceedings of ACL 2017",
    year = "2017",
}
```

## SingleRank

[Single Document Keyphrase Extraction Using Neighborhood Knowledge](https://ojs.aaai.org/index.php/AAAI/article/view/7798) (Wan & Xiao, 2008)

Extends TextRank with weighted edges based on co-occurrence frequency and cross-sentence windowing. Used by `SingleRank`.

```bibtex
@inproceedings{wan-xiao-2008-singlerank,
    title = "Single Document Keyphrase Extraction Using Neighborhood Knowledge",
    author = "Wan, Xiaojun and Xiao, Jianguo",
    booktitle = "Proceedings of the Twenty-Third AAAI Conference on Artificial Intelligence (AAAI 2008)",
    year = "2008",
    pages = "855--860",
}
```

## TopicRank

[TopicRank: Graph-Based Topic Ranking for Keyphrase Extraction](https://aclanthology.org/I13-1062/) (Bougouin et al., 2013)

Clusters candidate phrases into topics using hierarchical agglomerative clustering, then ranks topics as a whole. Used by `TopicRank`.

```bibtex
@inproceedings{bougouin-boudin-daille-2013-topicrank,
    title = "{T}opic{R}ank: Graph-Based Topic Ranking for Keyphrase Extraction",
    author = "Bougouin, Adrien and Boudin, Florian and Daille, B{\'e}atrice",
    booktitle = "Proceedings of the Sixth International Joint Conference on Natural Language Processing",
    year = "2013",
    pages = "543--551",
    publisher = "Asian Federation of Natural Language Processing",
}
```

## MultipartiteRank

[Unsupervised Keyphrase Extraction with Multipartite Graphs](https://aclanthology.org/N18-2105/) (Boudin, 2018)

Extends TopicRank by keeping individual candidates as graph nodes instead of collapsing topics, removing intra-topic edges to form a k-partite graph. Used by `MultipartiteRank`.

```bibtex
@inproceedings{boudin-2018-multipartiterank,
    title = "Unsupervised Keyphrase Extraction with Multipartite Graphs",
    author = "Boudin, Florian",
    booktitle = "Proceedings of the 2018 Conference of the North American Chapter of the Association for Computational Linguistics: Human Language Technologies (NAACL-HLT 2018)",
    year = "2018",
    pages = "667--672",
}
```

## Topical PageRank

[Topical Word Importance for Fast Keyphrase Extraction](https://aclanthology.org/W15-3605/) (Sterckx et al., 2015)

Biases PageRank toward topically important words using a personalization vector derived from topic models. Used by `TopicalPageRank`.

```bibtex
@inproceedings{sterckx-etal-2015-topical,
    title = "Topical Word Importance for Fast Keyphrase Extraction",
    author = "Sterckx, Lucas and Demeester, Thomas and Deleu, Johannes and Develder, Chris",
    booktitle = "Proceedings of the 24th International Conference on World Wide Web (Companion Volume)",
    year = "2015",
    pages = "121--122",
}
```

## BiasedTextRank

[BiasedTextRank: Unsupervised Graph-Based Content Extraction](https://aclanthology.org/2020.coling-main.144/) (Kazemi et al., 2020)

Steers extraction toward specific topics using focus terms and a bias weight in the PageRank personalization vector. Used by `BiasedTextRank`.

```bibtex
@inproceedings{kazemi-etal-2020-biasedtextrank,
    title = "Biased {T}ext{R}ank: Unsupervised Graph-Based Content Extraction",
    author = "Kazemi, Ashkan and P{\'e}rez-Rosas, Ver{\'o}nica and Mihalcea, Rada",
    booktitle = "Proceedings of the 28th International Conference on Computational Linguistics (COLING 2020)",
    year = "2020",
}
```
