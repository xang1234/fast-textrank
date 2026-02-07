//! Topical PageRank (SingleTPR) variant
//!
//! Topical PageRank (Sterckx et al., 2015) biases the random walk towards
//! topically important words. It combines SingleRank's graph construction
//! (weighted edges + cross-sentence windowing) with a personalized PageRank
//! whose teleport distribution reflects per-word topic importance.
//!
//! Users supply pre-computed topic weights (`lemma → weight`). Words absent
//! from the map receive a configurable minimum weight (default 0.0, matching
//! PKE's OOV behavior).

use crate::graph::builder::GraphBuilder;
use crate::graph::csr::CsrGraph;
use crate::pagerank::personalized::{topic_weight_personalization, PersonalizedPageRank};
use crate::phrase::extraction::{ExtractionResult, PhraseExtractor};
use crate::types::{Phrase, TextRankConfig, Token};
use std::collections::HashMap;

/// Topical PageRank implementation
#[derive(Debug)]
pub struct TopicalPageRank {
    config: TextRankConfig,
    /// Topic importance weights: lemma → weight
    topic_weights: HashMap<String, f64>,
    /// Weight assigned to words absent from topic_weights (PKE default: 0.0)
    min_weight: f64,
}

impl Default for TopicalPageRank {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicalPageRank {
    /// Create a new TopicalPageRank extractor with default config
    pub fn new() -> Self {
        Self {
            config: TextRankConfig::default(),
            topic_weights: HashMap::new(),
            min_weight: 0.0,
        }
    }

    /// Create with custom config
    pub fn with_config(config: TextRankConfig) -> Self {
        Self {
            config,
            topic_weights: HashMap::new(),
            min_weight: 0.0,
        }
    }

    /// Set topic importance weights (lemma → weight)
    pub fn with_topic_weights(mut self, weights: HashMap<String, f64>) -> Self {
        self.topic_weights = weights;
        self
    }

    /// Set the minimum weight for out-of-vocabulary words
    pub fn with_min_weight(mut self, min_weight: f64) -> Self {
        self.min_weight = min_weight;
        self
    }

    /// Extract keyphrases using Topical PageRank
    pub fn extract(&self, tokens: &[Token]) -> Vec<Phrase> {
        self.extract_with_info(tokens).phrases
    }

    /// Extract keyphrases with PageRank convergence information
    pub fn extract_with_info(&self, tokens: &[Token]) -> ExtractionResult {
        let include_pos = if self.config.include_pos.is_empty() {
            None
        } else {
            Some(self.config.include_pos.as_slice())
        };

        // SingleRank-style graph: weighted edges + cross-sentence windowing
        let builder = GraphBuilder::from_tokens_with_pos_and_boundaries(
            tokens,
            self.config.window_size,
            true,  // always weighted co-occurrence counts
            include_pos,
            self.config.use_pos_in_nodes,
            false, // ignore sentence boundaries
        );

        if builder.is_empty() {
            return ExtractionResult {
                phrases: Vec::new(),
                converged: true,
                iterations: 0,
            };
        }

        let graph = CsrGraph::from_builder(&builder);

        // Build personalization vector from topic weights
        let personalization = topic_weight_personalization(
            &self.topic_weights,
            &graph,
            &self.config.include_pos,
            self.config.use_pos_in_nodes,
            self.min_weight,
        );

        // Run Personalized PageRank
        let pagerank = PersonalizedPageRank::new()
            .with_damping(self.config.damping)
            .with_max_iterations(self.config.max_iterations)
            .with_threshold(self.config.convergence_threshold)
            .with_personalization(personalization)
            .run(&graph);

        let extractor = PhraseExtractor::with_config(self.config.clone());
        let phrases = extractor.extract(tokens, &graph, &pagerank);

        ExtractionResult {
            phrases,
            converged: pagerank.converged,
            iterations: pagerank.iterations,
        }
    }

    /// Get the current topic weights
    pub fn topic_weights(&self) -> &HashMap<String, f64> {
        &self.topic_weights
    }

    /// Get the minimum weight
    pub fn min_weight(&self) -> f64 {
        self.min_weight
    }
}

/// Convenience function to extract keyphrases using Topical PageRank
pub fn extract_keyphrases_topical(
    tokens: &[Token],
    config: &TextRankConfig,
    topic_weights: HashMap<String, f64>,
    min_weight: f64,
) -> Vec<Phrase> {
    TopicalPageRank::with_config(config.clone())
        .with_topic_weights(topic_weights)
        .with_min_weight(min_weight)
        .extract(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PosTag;

    fn make_token(text: &str, lemma: &str, pos: PosTag, sent: usize, idx: usize) -> Token {
        Token {
            text: text.to_string(),
            lemma: lemma.to_string(),
            pos,
            start: 0,
            end: text.len(),
            sentence_idx: sent,
            token_idx: idx,
            is_stopword: false,
        }
    }

    fn sample_tokens() -> Vec<Token> {
        vec![
            make_token("Machine", "machine", PosTag::Noun, 0, 0),
            make_token("learning", "learning", PosTag::Noun, 0, 1),
            make_token("is", "be", PosTag::Verb, 0, 2),
            make_token("artificial", "artificial", PosTag::Adjective, 0, 3),
            make_token("intelligence", "intelligence", PosTag::Noun, 0, 4),
            make_token("Deep", "deep", PosTag::Adjective, 1, 5),
            make_token("learning", "learning", PosTag::Noun, 1, 6),
            make_token("uses", "use", PosTag::Verb, 1, 7),
            make_token("neural", "neural", PosTag::Adjective, 1, 8),
            make_token("networks", "network", PosTag::Noun, 1, 9),
        ]
    }

    #[test]
    fn test_basic_extraction() {
        let tokens = sample_tokens();
        let mut weights = HashMap::new();
        weights.insert("machine".to_string(), 0.8);
        weights.insert("learning".to_string(), 0.6);

        let config = TextRankConfig::default().with_top_n(5);
        let result = TopicalPageRank::with_config(config)
            .with_topic_weights(weights)
            .extract_with_info(&tokens);

        assert!(!result.phrases.is_empty());
        assert!(result.converged);
    }

    #[test]
    fn test_empty_input() {
        let tokens: Vec<Token> = Vec::new();
        let config = TextRankConfig::default();
        let result = TopicalPageRank::with_config(config).extract_with_info(&tokens);

        assert!(result.phrases.is_empty());
        assert!(result.converged);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn test_empty_weights() {
        let tokens = sample_tokens();
        let config = TextRankConfig::default().with_top_n(5);

        // Empty topic weights → uniform min_weight → equivalent to uniform PPR
        let result = TopicalPageRank::with_config(config)
            .with_min_weight(1.0)
            .extract_with_info(&tokens);

        assert!(!result.phrases.is_empty());
    }

    #[test]
    fn test_topic_weights_bias_ranking() {
        let tokens = sample_tokens();
        let config = TextRankConfig::default().with_top_n(10);

        // Run with strong bias towards "neural" and "network"
        let mut neural_weights = HashMap::new();
        neural_weights.insert("neural".to_string(), 10.0);
        neural_weights.insert("network".to_string(), 10.0);

        let neural_result = TopicalPageRank::with_config(config.clone())
            .with_topic_weights(neural_weights)
            .extract_with_info(&tokens);

        // Run with strong bias towards "machine"
        let mut machine_weights = HashMap::new();
        machine_weights.insert("machine".to_string(), 10.0);

        let machine_result = TopicalPageRank::with_config(config)
            .with_topic_weights(machine_weights)
            .extract_with_info(&tokens);

        // "neural"/"network" should score higher in the neural-biased run
        let neural_score_biased = neural_result
            .phrases
            .iter()
            .find(|p| p.lemma.contains("neural") || p.lemma.contains("network"))
            .map(|p| p.score);
        let neural_score_machine = machine_result
            .phrases
            .iter()
            .find(|p| p.lemma.contains("neural") || p.lemma.contains("network"))
            .map(|p| p.score);

        if let (Some(biased), Some(unbiased)) = (neural_score_biased, neural_score_machine) {
            assert!(
                biased > unbiased,
                "neural/network should score higher with neural bias ({} vs {})",
                biased,
                unbiased
            );
        }
    }

    #[test]
    fn test_min_weight_affects_oov() {
        let tokens = sample_tokens();
        let config = TextRankConfig::default().with_top_n(10);

        // Only weight "machine", OOV gets 0
        let mut weights = HashMap::new();
        weights.insert("machine".to_string(), 10.0);

        let result_zero_min = TopicalPageRank::with_config(config.clone())
            .with_topic_weights(weights.clone())
            .with_min_weight(0.0)
            .extract_with_info(&tokens);

        // Same weights, but OOV gets 1.0
        let result_high_min = TopicalPageRank::with_config(config)
            .with_topic_weights(weights)
            .with_min_weight(1.0)
            .extract_with_info(&tokens);

        // With min_weight=0, "machine" should dominate more
        let machine_score_zero = result_zero_min
            .phrases
            .iter()
            .find(|p| p.lemma.contains("machine"))
            .map(|p| p.score)
            .unwrap_or(0.0);
        let machine_score_high = result_high_min
            .phrases
            .iter()
            .find(|p| p.lemma.contains("machine"))
            .map(|p| p.score)
            .unwrap_or(0.0);

        assert!(
            machine_score_zero >= machine_score_high,
            "machine should score at least as high with zero OOV weight ({} vs {})",
            machine_score_zero,
            machine_score_high
        );
    }

    #[test]
    fn test_convenience_function() {
        let tokens = sample_tokens();
        let config = TextRankConfig::default().with_top_n(5);
        let mut weights = HashMap::new();
        weights.insert("machine".to_string(), 0.8);

        let phrases = extract_keyphrases_topical(&tokens, &config, weights, 0.0);
        assert!(!phrases.is_empty());
    }
}
