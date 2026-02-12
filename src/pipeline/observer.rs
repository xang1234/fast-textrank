//! Pipeline observer — hooks for logging, profiling, and debugging.
//!
//! Observers receive notifications at stage boundaries without coupling to
//! stage logic. Use cases include timing stages, capturing intermediate
//! artifacts for debugging, and emitting structured telemetry.
//!
//! # Overview
//!
//! [`StageReport`] is a low-overhead metrics struct collected per stage by the
//! pipeline runner. Only `duration_us` is always populated; all other fields
//! are `Option` because different stages produce different metrics.

use crate::pipeline::artifacts::{CandidateSet, Graph, PhraseSet, RankOutput, TokenStream};
use std::time::{Duration, Instant};

// ============================================================================
// StageReport — per-stage metrics collected by the runner
// ============================================================================

/// Low-overhead per-stage metrics collected by the pipeline runner.
///
/// Every stage produces a `duration_us`. The remaining fields are populated
/// only by stages that have the relevant information:
///
/// | Field        | Populated by            |
/// |--------------|-------------------------|
/// | `nodes`      | GraphBuilder            |
/// | `edges`      | GraphBuilder            |
/// | `iterations` | Ranker                  |
/// | `converged`  | Ranker                  |
/// | `residual`   | Ranker                  |
///
/// # Construction
///
/// Use [`StageReport::new`] (duration only) or [`StageReportBuilder`] for a
/// fluent construction pattern:
///
/// ```
/// # use rapid_textrank::pipeline::observer::{StageReport, StageReportBuilder};
/// # use std::time::Duration;
/// let report = StageReportBuilder::new(Duration::from_micros(420))
///     .nodes(128)
///     .edges(512)
///     .build();
///
/// assert_eq!(report.duration_us(), 420);
/// assert_eq!(report.nodes(), Some(128));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StageReport {
    /// Wall-clock duration of the stage in microseconds.
    duration_us: u64,
    /// Number of graph nodes produced (GraphBuilder).
    nodes: Option<usize>,
    /// Number of graph edges produced (GraphBuilder).
    edges: Option<usize>,
    /// Number of iterations performed (Ranker).
    iterations: Option<u32>,
    /// Whether the ranker converged within threshold (Ranker).
    converged: Option<bool>,
    /// Final convergence residual / L1-norm delta (Ranker).
    residual: Option<f64>,
}

impl StageReport {
    /// Create a report with only the stage duration.
    #[inline]
    pub fn new(duration: Duration) -> Self {
        Self {
            duration_us: duration.as_micros() as u64,
            nodes: None,
            edges: None,
            iterations: None,
            converged: None,
            residual: None,
        }
    }

    /// Wall-clock duration of the stage in microseconds.
    #[inline]
    pub fn duration_us(&self) -> u64 {
        self.duration_us
    }

    /// Duration as a [`Duration`] value.
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_micros(self.duration_us)
    }

    /// Duration in milliseconds as `f64` (for display / JSON serialization).
    #[inline]
    pub fn duration_ms(&self) -> f64 {
        self.duration_us as f64 / 1000.0
    }

    /// Number of graph nodes, if reported.
    #[inline]
    pub fn nodes(&self) -> Option<usize> {
        self.nodes
    }

    /// Number of graph edges, if reported.
    #[inline]
    pub fn edges(&self) -> Option<usize> {
        self.edges
    }

    /// Number of ranker iterations, if reported.
    #[inline]
    pub fn iterations(&self) -> Option<u32> {
        self.iterations
    }

    /// Whether the ranker converged, if reported.
    #[inline]
    pub fn converged(&self) -> Option<bool> {
        self.converged
    }

    /// Final convergence residual (L1-norm delta), if reported.
    #[inline]
    pub fn residual(&self) -> Option<f64> {
        self.residual
    }
}

// ============================================================================
// StageReportBuilder — fluent construction
// ============================================================================

/// Fluent builder for [`StageReport`].
///
/// Construct via [`StageReportBuilder::new`] with the stage duration, then
/// chain optional field setters.
///
/// ```
/// # use rapid_textrank::pipeline::observer::StageReportBuilder;
/// # use std::time::Duration;
/// let report = StageReportBuilder::new(Duration::from_millis(3))
///     .iterations(42)
///     .converged(true)
///     .residual(1e-7)
///     .build();
///
/// assert_eq!(report.iterations(), Some(42));
/// assert_eq!(report.converged(), Some(true));
/// ```
pub struct StageReportBuilder {
    report: StageReport,
}

impl StageReportBuilder {
    /// Start building a report with the given stage duration.
    #[inline]
    pub fn new(duration: Duration) -> Self {
        Self {
            report: StageReport::new(duration),
        }
    }

    /// Record the number of graph nodes.
    #[inline]
    pub fn nodes(mut self, n: usize) -> Self {
        self.report.nodes = Some(n);
        self
    }

    /// Record the number of graph edges.
    #[inline]
    pub fn edges(mut self, n: usize) -> Self {
        self.report.edges = Some(n);
        self
    }

    /// Record the number of ranker iterations.
    #[inline]
    pub fn iterations(mut self, n: u32) -> Self {
        self.report.iterations = Some(n);
        self
    }

    /// Record whether the ranker converged.
    #[inline]
    pub fn converged(mut self, c: bool) -> Self {
        self.report.converged = Some(c);
        self
    }

    /// Record the final convergence residual.
    #[inline]
    pub fn residual(mut self, r: f64) -> Self {
        self.report.residual = Some(r);
        self
    }

    /// Consume the builder and return the finished [`StageReport`].
    #[inline]
    pub fn build(self) -> StageReport {
        self.report
    }
}

// ============================================================================
// PipelineObserver — trait for stage-boundary hooks
// ============================================================================

/// Receives notifications at pipeline stage boundaries.
///
/// All methods have default no-op implementations, so implementors only
/// override the callbacks they need. The runner calls methods in this order
/// for each stage:
///
/// 1. [`on_stage_start`](PipelineObserver::on_stage_start) — before execution
/// 2. (stage runs)
/// 3. [`on_stage_end`](PipelineObserver::on_stage_end) — after execution, with metrics
///
/// Artifact callbacks (`on_tokens`, `on_candidates`, etc.) are called by the
/// runner *after* the producing stage completes and *before* the next stage
/// starts, giving observers a read-only view of each intermediate result.
///
/// # Stage names
///
/// Stage names are `&'static str` constants, known at compile time:
///
/// | Constant | Value |
/// |---|---|
/// | [`STAGE_PREPROCESS`] | `"preprocess"` |
/// | [`STAGE_CANDIDATES`] | `"candidates"` |
/// | [`STAGE_GRAPH`] | `"graph"` |
/// | [`STAGE_GRAPH_TRANSFORM`] | `"graph_transform"` |
/// | [`STAGE_TELEPORT`] | `"teleport"` |
/// | [`STAGE_RANK`] | `"rank"` |
/// | [`STAGE_PHRASES`] | `"phrases"` |
/// | [`STAGE_FORMAT`] | `"format"` |
///
/// # Zero overhead
///
/// When using [`NoopObserver`] (the default), the compiler eliminates all
/// callback calls via monomorphization — there is no runtime cost.
pub trait PipelineObserver {
    /// Called before a stage begins execution.
    fn on_stage_start(&mut self, _stage: &'static str) {}

    /// Called after a stage completes, with its [`StageReport`] metrics.
    fn on_stage_end(&mut self, _stage: &'static str, _report: &StageReport) {}

    /// Called after the Preprocessor stage with the (possibly mutated) token stream.
    fn on_tokens(&mut self, _tokens: &TokenStream) {}

    /// Called after the CandidateSelector stage with the selected candidates.
    fn on_candidates(&mut self, _candidates: &CandidateSet) {}

    /// Called after the GraphBuilder (and optional GraphTransform) stage.
    fn on_graph(&mut self, _graph: &Graph) {}

    /// Called after the Ranker stage with scores and convergence info.
    fn on_rank(&mut self, _rank: &RankOutput) {}

    /// Called after the PhraseBuilder stage with the assembled phrases.
    fn on_phrases(&mut self, _phrases: &PhraseSet) {}
}

/// Well-known stage name constants used in observer callbacks.
pub const STAGE_PREPROCESS: &str = "preprocess";
/// Stage name for candidate selection.
pub const STAGE_CANDIDATES: &str = "candidates";
/// Stage name for graph construction.
pub const STAGE_GRAPH: &str = "graph";
/// Stage name for graph transforms.
pub const STAGE_GRAPH_TRANSFORM: &str = "graph_transform";
/// Stage name for teleport vector construction.
pub const STAGE_TELEPORT: &str = "teleport";
/// Stage name for ranking (PageRank).
pub const STAGE_RANK: &str = "rank";
/// Stage name for phrase building.
pub const STAGE_PHRASES: &str = "phrases";
/// Stage name for result formatting.
pub const STAGE_FORMAT: &str = "format";

// ============================================================================
// NoopObserver — zero-overhead default
// ============================================================================

/// No-op observer — all callbacks are empty.
///
/// When the runner is generic over `impl PipelineObserver`, using
/// `NoopObserver` causes the compiler to eliminate all callback call-sites
/// entirely via monomorphization.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopObserver;

impl PipelineObserver for NoopObserver {}

// ============================================================================
// StageTimingObserver — collects per-stage timing reports
// ============================================================================

/// Collects [`StageReport`]s for every stage into a `Vec`.
///
/// This is the simplest useful observer: it records the report from each
/// `on_stage_end` call so you can inspect timings after the pipeline run.
///
/// ```
/// # use rapid_textrank::pipeline::observer::{StageTimingObserver, PipelineObserver, StageReport, STAGE_RANK};
/// # use std::time::Duration;
/// let mut obs = StageTimingObserver::new();
///
/// // Simulate what the runner does:
/// obs.on_stage_start(STAGE_RANK);
/// let report = StageReport::new(Duration::from_millis(5));
/// obs.on_stage_end(STAGE_RANK, &report);
///
/// assert_eq!(obs.reports().len(), 1);
/// assert_eq!(obs.reports()[0].0, STAGE_RANK);
/// assert_eq!(obs.reports()[0].1.duration_us(), 5_000);
/// ```
#[derive(Debug, Clone, Default)]
pub struct StageTimingObserver {
    reports: Vec<(&'static str, StageReport)>,
}

impl StageTimingObserver {
    /// Create an empty observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// The collected `(stage_name, report)` pairs in execution order.
    pub fn reports(&self) -> &[(&'static str, StageReport)] {
        &self.reports
    }

    /// Total wall-clock duration across all recorded stages.
    pub fn total_duration(&self) -> Duration {
        let total_us: u64 = self.reports.iter().map(|(_, r)| r.duration_us()).sum();
        Duration::from_micros(total_us)
    }

    /// Total wall-clock duration in milliseconds.
    pub fn total_duration_ms(&self) -> f64 {
        self.total_duration().as_micros() as f64 / 1000.0
    }
}

impl PipelineObserver for StageTimingObserver {
    fn on_stage_end(&mut self, stage: &'static str, report: &StageReport) {
        self.reports.push((stage, report.clone()));
    }
}

// ============================================================================
// StageClock — lightweight timer helper
// ============================================================================

/// Lightweight timer for measuring stage durations.
///
/// Call [`StageClock::start`] before a stage, then [`StageClock::elapsed`]
/// after it completes to get the duration for [`StageReport`] construction.
///
/// ```
/// # use rapid_textrank::pipeline::observer::StageClock;
/// let clock = StageClock::start();
/// // ... do work ...
/// let duration = clock.elapsed();
/// ```
pub struct StageClock {
    start: Instant,
}

impl StageClock {
    /// Start the clock.
    #[inline]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Elapsed time since the clock was started.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::artifacts::{CandidateSet, Graph, PhraseSet, RankOutput};

    // -- StageReport tests --------------------------------------------------

    #[test]
    fn test_new_report_has_only_duration() {
        let report = StageReport::new(Duration::from_micros(500));
        assert_eq!(report.duration_us(), 500);
        assert!(report.nodes().is_none());
        assert!(report.edges().is_none());
        assert!(report.iterations().is_none());
        assert!(report.converged().is_none());
        assert!(report.residual().is_none());
    }

    #[test]
    fn test_duration_ms_conversion() {
        let report = StageReport::new(Duration::from_micros(1_500));
        assert!((report.duration_ms() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_duration_roundtrip() {
        let original = Duration::from_micros(42_000);
        let report = StageReport::new(original);
        assert_eq!(report.duration(), original);
    }

    #[test]
    fn test_builder_graph_fields() {
        let report = StageReportBuilder::new(Duration::from_millis(3))
            .nodes(128)
            .edges(512)
            .build();

        assert_eq!(report.duration_us(), 3_000);
        assert_eq!(report.nodes(), Some(128));
        assert_eq!(report.edges(), Some(512));
        assert!(report.iterations().is_none());
        assert!(report.converged().is_none());
        assert!(report.residual().is_none());
    }

    #[test]
    fn test_builder_ranker_fields() {
        let report = StageReportBuilder::new(Duration::from_millis(10))
            .iterations(42)
            .converged(true)
            .residual(1e-7)
            .build();

        assert_eq!(report.iterations(), Some(42));
        assert_eq!(report.converged(), Some(true));
        assert!((report.residual().unwrap() - 1e-7).abs() < f64::EPSILON);
        assert!(report.nodes().is_none());
    }

    #[test]
    fn test_builder_all_fields() {
        let report = StageReportBuilder::new(Duration::from_micros(999))
            .nodes(64)
            .edges(256)
            .iterations(100)
            .converged(false)
            .residual(0.01)
            .build();

        assert_eq!(report.duration_us(), 999);
        assert_eq!(report.nodes(), Some(64));
        assert_eq!(report.edges(), Some(256));
        assert_eq!(report.iterations(), Some(100));
        assert_eq!(report.converged(), Some(false));
        assert!((report.residual().unwrap() - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn test_report_clone_and_eq() {
        let report = StageReportBuilder::new(Duration::from_millis(5))
            .nodes(10)
            .build();
        let cloned = report.clone();
        assert_eq!(report, cloned);
    }

    #[test]
    fn test_stage_clock_measures_time() {
        let clock = StageClock::start();
        // Spin briefly to ensure non-zero elapsed time.
        std::thread::sleep(Duration::from_millis(1));
        let elapsed = clock.elapsed();
        assert!(elapsed >= Duration::from_millis(1));
    }

    #[test]
    fn test_zero_duration_report() {
        let report = StageReport::new(Duration::ZERO);
        assert_eq!(report.duration_us(), 0);
        assert!((report.duration_ms() - 0.0).abs() < f64::EPSILON);
    }

    // -- PipelineObserver trait tests ----------------------------------------

    #[test]
    fn test_noop_observer_compiles_and_runs() {
        let mut obs = NoopObserver;
        obs.on_stage_start(STAGE_PREPROCESS);
        obs.on_stage_end(STAGE_PREPROCESS, &StageReport::new(Duration::from_millis(1)));
        // No panics, no side effects — that's the contract.
    }

    #[test]
    fn test_noop_observer_artifact_callbacks() {
        use crate::types::StringPool;

        let mut obs = NoopObserver;
        let ts = TokenStream::new(vec![], StringPool::new());
        let cs = CandidateSet::empty();
        let graph = Graph::empty();
        let rank = RankOutput::new(vec![], true, 0, 0.0);
        let phrases = PhraseSet::empty();

        obs.on_tokens(&ts);
        obs.on_candidates(&cs);
        obs.on_graph(&graph);
        obs.on_rank(&rank);
        obs.on_phrases(&phrases);
    }

    #[test]
    fn test_stage_timing_observer_collects_reports() {
        let mut obs = StageTimingObserver::new();
        assert!(obs.reports().is_empty());

        let r1 = StageReportBuilder::new(Duration::from_millis(2))
            .nodes(10)
            .edges(20)
            .build();
        let r2 = StageReportBuilder::new(Duration::from_millis(5))
            .iterations(42)
            .converged(true)
            .build();

        obs.on_stage_start(STAGE_GRAPH);
        obs.on_stage_end(STAGE_GRAPH, &r1);
        obs.on_stage_start(STAGE_RANK);
        obs.on_stage_end(STAGE_RANK, &r2);

        assert_eq!(obs.reports().len(), 2);
        assert_eq!(obs.reports()[0].0, STAGE_GRAPH);
        assert_eq!(obs.reports()[0].1.nodes(), Some(10));
        assert_eq!(obs.reports()[1].0, STAGE_RANK);
        assert_eq!(obs.reports()[1].1.iterations(), Some(42));
    }

    #[test]
    fn test_stage_timing_observer_total_duration() {
        let mut obs = StageTimingObserver::new();
        obs.on_stage_end(STAGE_PREPROCESS, &StageReport::new(Duration::from_millis(1)));
        obs.on_stage_end(STAGE_CANDIDATES, &StageReport::new(Duration::from_millis(2)));
        obs.on_stage_end(STAGE_GRAPH, &StageReport::new(Duration::from_millis(3)));

        assert_eq!(obs.total_duration(), Duration::from_millis(6));
        assert!((obs.total_duration_ms() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stage_timing_observer_empty_total() {
        let obs = StageTimingObserver::new();
        assert_eq!(obs.total_duration(), Duration::ZERO);
    }

    /// Custom observer that records stage names for testing generic dispatch.
    struct RecordingObserver {
        started: Vec<&'static str>,
        ended: Vec<&'static str>,
    }

    impl RecordingObserver {
        fn new() -> Self {
            Self {
                started: vec![],
                ended: vec![],
            }
        }
    }

    impl PipelineObserver for RecordingObserver {
        fn on_stage_start(&mut self, stage: &'static str) {
            self.started.push(stage);
        }
        fn on_stage_end(&mut self, stage: &'static str, _report: &StageReport) {
            self.ended.push(stage);
        }
    }

    #[test]
    fn test_custom_observer_receives_callbacks() {
        let mut obs = RecordingObserver::new();

        obs.on_stage_start(STAGE_CANDIDATES);
        obs.on_stage_end(STAGE_CANDIDATES, &StageReport::new(Duration::from_millis(1)));
        obs.on_stage_start(STAGE_GRAPH);
        obs.on_stage_end(STAGE_GRAPH, &StageReport::new(Duration::from_millis(2)));

        assert_eq!(obs.started, vec![STAGE_CANDIDATES, STAGE_GRAPH]);
        assert_eq!(obs.ended, vec![STAGE_CANDIDATES, STAGE_GRAPH]);
    }

    #[test]
    fn test_observer_generic_dispatch() {
        // Verify the trait works through generic function boundaries.
        fn run_with_observer(obs: &mut impl PipelineObserver) {
            obs.on_stage_start(STAGE_RANK);
            obs.on_stage_end(STAGE_RANK, &StageReport::new(Duration::from_millis(10)));
        }

        let mut timing = StageTimingObserver::new();
        run_with_observer(&mut timing);
        assert_eq!(timing.reports().len(), 1);

        let mut noop = NoopObserver;
        run_with_observer(&mut noop); // compiles and runs — zero overhead
    }

    #[test]
    fn test_stage_name_constants_are_distinct() {
        let names = [
            STAGE_PREPROCESS,
            STAGE_CANDIDATES,
            STAGE_GRAPH,
            STAGE_GRAPH_TRANSFORM,
            STAGE_TELEPORT,
            STAGE_RANK,
            STAGE_PHRASES,
            STAGE_FORMAT,
        ];
        // All names should be unique.
        let mut sorted = names.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "Stage name constants must be unique");
    }
}
