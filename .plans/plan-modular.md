# plan-unify.md — Unified Modular Pipeline Plan for rapid-textrank

This document is a revised, unified, engineering-focused plan to make **rapid-textrank**’s extraction flow an explicitly **modular pipeline**—internally (Rust core) and publicly (power-user API + docs)—while keeping today’s “easy” API intact and preserving the library’s performance properties (CSR graph, interning, Rayon, etc.).

The plan is grounded in how the library already describes itself and its algorithm “variants”:

- Classic flow: **graph build → PageRank → phrase extraction**
- Existing variants are already stage mutations:
  - **PositionRank**: teleport/personalization depends on early positions
  - **BiasedTextRank**: teleport depends on focus terms
  - **SingleRank**: graph construction changes (weighted edges + cross-sentence windowing)
  - **TopicalPageRank**: personalization vector from per-lemma topic weights
  - **TopicRank**: candidate clustering + topic graph + representative selection
  - **MultipartiteRank**: clustering + intra-topic edge removal + alpha boost + PageRank

The goal is to formalize this as an explicit, composable pipeline—**without regressing performance**—and to expose intermediate artifacts safely and cheaply (opt-in debug).

---

## Goals

1. **Explicit internal modular pipeline** with first-class artifacts and stage interfaces.
2. **No breaking changes** for existing APIs:
   - `extract_keywords()` convenience
   - extractor classes (BaseTextRank, PositionRank, BiasedTextRank, SingleRank, TopicalPageRank, TopicRank/MultipartiteRank where applicable)
   - existing JSON input format (variant + config)
3. **Power-user modularity** via an optional JSON `pipeline` object (PipelineSpec v1), enabling new combinations (e.g., SingleRank graph + PositionRank teleport).
4. **Reliability and production readiness**:
   - first-class validation and stable errors
   - deterministic mode (reproducible outputs)
   - runtime safety rails (limits on tokens/nodes/edges)
   - capability discovery (what this build supports)
5. **Performance**:
   - keep CSR graph for cache-friendly iteration
   - avoid extra allocations by default (borrowed views, reusable workspace)
   - release Python GIL during extraction
6. **Introspection**:
   - observer hooks with minimal overhead
   - stage-level timing metrics and optional `tracing` instrumentation

---

## Non-goals

- Breaking changes to the existing “easy” API.
- Exposing Rust trait objects to Python directly (not ergonomic); JSON PipelineSpec is the primary cross-language contract.
- Forcing debug allocations on the hot path (debug is opt-in and cost-controlled).
- Shipping every possible module combination on day one; start with existing variants and extend gradually.

---

## 0) Validation + Error Semantics as a First-Class Contract (Reliability Gate)

Modularity (especially via JSON PipelineSpec) increases the probability of invalid configurations. Validation and error reporting must be part of the public contract so failures are explicit, actionable, and stable.

### Unified error model (Rust + JSON + Python)
Define a single error taxonomy used end-to-end:

- `PipelineSpecError` — schema/validation problems (build time)
- `PipelineRuntimeError` — failures during execution (stage failures)

Each error should include:
- `code`: stable string/enum (e.g., `missing_stage`, `invalid_combo`, `module_unavailable`, `limit_exceeded`)
- `path`: JSON pointer-style path (e.g., `/pipeline/modules/rank/type`)
- `stage`: stage name if runtime error
- `message`: human-readable
- `hint`: optional next-step fix

### Validate-only mode
Support preflight validation without running extraction:
- JSON: `{ "pipeline": {...}, "validate_only": true }`
- Rust: `PipelineBuilder::validate(spec, cfg)`
- Python: `Extractor.validate_pipeline_spec(...)` or `extract_from_json(validate_only=True)`

### Example validation rules
- `personalized_pagerank` **requires** `teleport` (or explicitly `uniform` teleport)
- `topic_graph` requires `clustering` + `phrase_candidates`
- `remove_intra_cluster_edges` requires cluster assignments to exist
- unknown fields are errors when `strict=true` (warnings in non-strict)
- enforce runtime limits (max tokens/nodes/edges) early (fail fast)

---

## 1) Make Intermediate Artifacts First-Class Types (Low-Allocation, Stable)

Today, “pipeline stages” are mostly conceptual. Make them explicit via stable, low-allocation structs. Preserve performance by using ID-based internal representation (interning / integer IDs), only materializing strings at the formatting boundary.

### Core shared artifacts

**Token / TokenStream**
- Must match JSON interface semantics:
  - text, lemma, pos, offsets, sentence/token indices, is_stopword
- Interned string IDs for text/lemma/pos where possible.

**CandidateSet**
- Word-node candidates (Base/Position/Biased/Single/TopicalPageRank)
- Phrase candidates (TopicRank/MultipartiteRank families)

**Graph**
- CSR-backed adjacency + weights
- NodeIndex mapping from internal node IDs → CSR indices

**RankOutput**
- `scores`, `converged`, `iterations`
- optionally residual/diagnostics (behind debug)

**PhraseSet (pre-format)**
- phrase IDs / lemma IDs / surface forms (optionally)
- score, count
- optional spans/positions (debug)

**FormattedResult**
- today’s `TextRankResult` and `Phrase` objects (Python + JSON consistent)
- optional `debug` object appended (opt-in)

### Borrowed vs Owned (allocation control)
Model borrowed views explicitly to keep the hot path cheap:

- `TokenStreamRef<'a>`, `CandidateSetRef<'a>`, `PhraseSetRef<'a>`: stage interfaces accept refs
- Owned artifacts only where necessary: `Graph`, `RankOutput`, final `PhraseSet`

### PipelineWorkspace (reusable buffers)
Add a per-extractor reusable workspace to reduce allocator churn across repeated calls (common in Python):

- edge buffers, adjacency scratch
- pagerank score vectors + normalization scratch
- phrase grouping scratch

Debug snapshots are copy-on-demand and explicitly requested.

### Performance invariants
- Artifacts are ID-based internally; strings materialize at formatting boundary.
- Pass references/slices; avoid cloning large arrays unless debug explicitly enabled.
- Keep CSR graph as the underlying storage to preserve cache-friendly PageRank iteration.

---

## 2) Determinism Modes + Stable Tie-Breakers (Reproducibility)

Parallelism + hash iteration order + floating point reduction order can cause non-reproducible outputs. Provide an explicit determinism mode.

### Determinism modes
- **Default**: fastest (parallel reductions permitted)
- **Deterministic** (opt-in): stable results across runs/machines

### Deterministic output contract (tie-breakers)
When scores tie within epsilon, apply stable tie-breakers:
1) score desc
2) earliest first occurrence asc
3) shorter phrase length asc
4) lemma (or normalized surface) lexicographic asc

### Deterministic construction requirements
- CSR construction order must be stable under parallelism
- avoid `HashMap` iteration order affecting output (sort keys or use stable maps)
- deterministic reductions where relevant

---

## 3) Define Explicit Stage Traits (Rust Core)

Create `src/pipeline/` and define stage interfaces. Use static composition internally (fastest), and expose a config-driven dynamic pipeline builder publicly (JSON-friendly; Python-friendly).

### Stage traits (core boundaries)

```rust
// 0) Optional: preprocessing / normalization (canonicalize across token sources)
pub trait Preprocessor {
    fn preprocess(&self, tokens: &mut TokenStream, cfg: &TextRankConfig);
}

// 1) Token stream → candidate selection / filters
pub trait CandidateSelector {
    fn select(&self, tokens: TokenStreamRef<'_>, cfg: &TextRankConfig) -> CandidateSet;
}

// 2) Graph builder (windowing, weighting, cross-sentence behavior)
pub trait GraphBuilder {
    fn build(&self, tokens: TokenStreamRef<'_>, candidates: CandidateSetRef<'_>, cfg: &TextRankConfig) -> Graph;
}

// 2a) Optional: graph transforms (MultipartiteRank, future variants)
pub trait GraphTransform {
    fn transform(&self, graph: &mut Graph, tokens: TokenStreamRef<'_>, candidates: CandidateSetRef<'_>, cfg: &TextRankConfig);
}

// 3a) Optional: teleport/personalization builder
pub trait TeleportBuilder {
    fn build(&self, tokens: TokenStreamRef<'_>, candidates: CandidateSetRef<'_>, cfg: &TextRankConfig)
        -> Option<TeleportVector>;
}

// 3) Ranker (PageRank / Personalized PageRank)
pub trait Ranker {
    fn rank(&self, graph: &Graph, teleport: Option<&TeleportVector>, cfg: &TextRankConfig)
        -> RankOutput;
}

// 4) Phrase builder (chunking rules, stopword boundaries, grouping strategy)
pub trait PhraseBuilder {
    fn build(&self, tokens: TokenStreamRef<'_>, candidates: CandidateSetRef<'_>, ranks: &RankOutput, cfg: &TextRankConfig)
        -> PhraseSet;
}

// 5) Result formatting (scores, positions, counts, topics, debug info)
pub trait ResultFormatter {
    fn format(&self, phrases: PhraseSet, ranks: &RankOutput, debug: Option<DebugInfo>, cfg: &TextRankConfig)
        -> TextRankResult;
}
```

### Why TeleportBuilder matters
It cleanly models PositionRank / BiasedTextRank / TopicalPageRank as “same graph, different teleport distribution.”

### Why Preprocessor matters
It centralizes normalization differences between:
- built-in tokenizer (Unicode-aware, may ignore some tokens like emoji), and
- spaCy tokens / JSON-provided tokens,
without duplicating normalization rules across CandidateSelector/GraphBuilder/PhraseBuilder.

---

## 4) Pipeline Runner + Observer Hooks + Stage Metrics + Optional Tracing

Introduce:

- `Pipeline`: composition of stages
- `PipelineRunner`: executes stages in order
- `PipelineObserver`: optional, for power-user introspection + tests
- `StageReport`: low-overhead metrics per stage

### Observer interface

```rust
pub struct StageReport {
    pub duration_ms: u64,
    pub nodes: Option<usize>,
    pub edges: Option<usize>,
    pub iterations: Option<u32>,
    pub converged: Option<bool>,
    pub residual: Option<f64>, // optional/debug
}

pub trait PipelineObserver {
    fn on_stage_start(&mut self, _stage: &'static str) {}
    fn on_stage_end(&mut self, _stage: &'static str, _report: &StageReport) {}

    fn on_tokens(&mut self, _tokens: &TokenStream) {}
    fn on_candidates(&mut self, _candidates: &CandidateSet) {}
    fn on_graph(&mut self, _graph: &Graph) {}
    fn on_rank(&mut self, _rank: &RankOutput) {}
    fn on_phrases(&mut self, _phrases: &PhraseSet) {}
}
```

### Instrumentation
- Add stage-level timing in `PipelineRunner` by default (cheap).
- Add feature-flagged `tracing` spans around stage calls for structured debugging.

---

## 5) Variant Composition Becomes Explicit (and Testable)

Implement each algorithm variant as a pipeline spec (composition of modules), replacing scattered conditionals.

### Word-graph family

**BaseTextRank**
- Preprocessor: default normalization (optional)
- CandidateSelector: POS + stopword filtering (config-driven)
- GraphBuilder: windowed co-occurrence; sentence-bounded by default
- Teleport: uniform (None / implicit)
- Ranker: PageRank
- PhraseBuilder: noun-chunking + aggregation + grouping
- Formatter: standard

**PositionRank**
- Same as Base, but TeleportBuilder = position-weighted

**BiasedTextRank**
- Same as Base, but TeleportBuilder = focus-terms-biased

**SingleRank**
- GraphBuilder changes: weighted edges by co-occurrence count + cross-sentence windowing
- Ranker: PageRank

**TopicalPageRank**
- GraphBuilder: SingleRank-like
- TeleportBuilder: topic weights (+ min weight)
- Ranker: Personalized PageRank

### Topic/phrase family

**TopicRank (JSON-only)**
- Candidates: noun chunks (phrase candidates)
- Clustering: HAC/Jaccard
- Graph: topic-level graph (topics as nodes)
- Ranker: PageRank
- PhraseBuilder: representative selection per top topic
- Formatter: standard

**MultipartiteRank**
- Candidates + clustering
- Graph: candidate graph
- GraphTransforms: remove intra-topic edges; alpha boost (position preference)
- Ranker: PageRank
- PhraseBuilder: return top candidates directly
- Formatter: standard

This composition matrix is the design truth: variants differ by swapping stages.

---

## 6) Python Performance Milestone: Release the GIL During Extraction

Python users often run extraction across many documents. Holding the GIL during the entire run limits throughput and reduces the real-world benefit of Rayon inside Rust.

### Plan
- Wrap `PipelineRunner` execution with `Python::allow_threads(|| ...)` in pyo3.
- Ensure any Python callbacks (if ever added) reacquire the GIL; prefer Rust-side observers for performance.
- Add runtime controls for Rayon:
  - `max_threads`
  - `single_thread` mode
  - optional per-extractor `rayon::ThreadPool` to avoid global contention

---

## 7) Public Modularity Without Breaking Normal Users

Two public layers:

### A) Keep existing API exactly as-is
- `extract_keywords()` convenience
- extractor classes (BaseTextRank, PositionRank, BiasedTextRank, SingleRank, TopicalPageRank, MultipartiteRank)
- JSON variant dispatch remains supported

### B) Add explicit pipeline public API (Rust + JSON first; Python later)
Python can’t pass Rust trait objects easily. The practical cross-language approach is a JSON PipelineSpec.

- Rust: `rapid_textrank::pipeline::PipelineBuilder`
- `PipelineSpec` with enums for stage choices
- `Pipeline::run(input, cfg, observer)` returning `TextRankResult` (+ optional debug)

---

## 8) PipelineSpec v1 (Concrete JSON Shape)

This extends the current JSON `JsonDocument` shape (`tokens`, optional `variant`, optional `config`) with an optional `pipeline`.

### 8.1 Extend JsonDocument with optional `pipeline`

Current input schema:
```json
{
  "tokens": [ ... ],
  "variant": "textrank",
  "config": { ... }
}
```

Proposed schema (backward compatible):
```json
{
  "tokens": [ ... ],
  "variant": "textrank",
  "config": { ... },

  "pipeline": "textrank"
  // OR a PipelineSpec object (below)
}
```

Rules:
- If `pipeline` is missing → behave exactly like today (use `variant` + `config`)
- If `pipeline` is a string → treat as preset/alias (same accepted strings as `variant`)
- If `pipeline` is an object → build a pipeline from explicit modules
- If `validate_only=true` → validate and return a validation report, do not run extraction

### 8.2 Precedence + merging model (explicit to avoid ambiguity)

When building an effective pipeline configuration, apply:

1) `pipeline.modules.*` explicit fields (highest priority)  
2) `config` fields (TextRankConfig / JSON config)  
3) preset defaults (lowest priority)

Rule of thumb:
- Module specs omit fields when they want “whatever config says”
- Only specify module fields to override config behavior

### 8.3 Top-level PipelineSpec v1
```json
{
  "v": 1,
  "preset": "textrank",
  "modules": { ... },
  "expose": { ... },
  "runtime": { ... },
  "strict": true
}
```

Fields:
- `v` (required): version (start at 1)
- `preset` (optional): shorthand starting point (same strings as `variant`)
- `modules` (optional if preset provided): explicit module selections
- `expose` (optional): request intermediate artifacts / debug stats in response
- `runtime` (optional): non-algorithm execution controls (production hardening)
- `strict` (optional, default false): unknown fields/types error if true; warn if false

### 8.4 Stage keys and module specs
Stable stage keys (public contract):
```json
"modules": {
  "candidates": { ... },
  "graph": { ... },
  "rank": { ... },
  "phrases": { ... },
  "format": { ... },

  "preprocess": { ... },        // optional
  "teleport": { ... },          // optional
  "clustering": { ... },         // optional
  "graph_transforms": [ ... ]    // optional
}
```

Each module is a tagged object:
```json
{ "type": "some_module_type", "...params": "..." }
```

Serde-friendly in Rust: `#[serde(tag="type", rename_all="snake_case")]`.

### 8.5 `runtime` (production execution controls)
Example:
```json
"runtime": {
  "max_threads": 8,
  "deterministic": true,
  "max_tokens": 200000,
  "max_nodes": 200000,
  "max_edges": 5000000,
  "max_debug_top_k": 200
}
```

### 8.6 `expose` (debug/introspection requests)
Example:
```json
"expose": {
  "node_scores": { "top_k": 50 },
  "graph_stats": true,
  "pagerank": { "residuals": true },
  "clusters": true,
  "stage_timings": true
}
```

Attach these under a single optional key in the response (e.g., `"debug": {...}`) so normal results stay unchanged.

---

## 9) Capability Discovery and Module Availability

PipelineSpec must handle:
- module types not compiled in this build
- module types available in Rust but not in Python wheels
- version mismatches across client/server

Add a capability discovery endpoint/command:

- Rust: `rapid_textrank::capabilities() -> Capabilities`
- JSON: `{ "capabilities": true }` request or a dedicated CLI command
- Output:
  - supported PipelineSpec versions
  - available module types per stage
  - build flags / features enabled

Validation should return `module_unavailable` with a clear message when a module is requested but not available.

---

## 10) Implementation Plan (Safe Refactors in PR-Sized Phases)

### Phase 0 — Architecture doc, invariants, validation spec
Deliverables:
- `docs/architecture/pipeline.md`:
  - stage diagram
  - artifact definitions (borrowed vs owned)
  - variant composition matrix
  - determinism modes + tie-breakers
  - validation rules + error codes
- Tests:
  - BaseTextRank goldens matching current behavior
  - deterministic-mode goldens for CI stability
  - validation tests for invalid PipelineSpecs (missing stages, illegal combos)

### Phase 1 — Pipeline skeleton (no functional change)
- Create `src/pipeline/`:
  - `artifacts.rs`, `traits.rs`, `runner.rs`, `observer.rs`, `variants.rs`
- Add adapters wrapping existing implementation paths:
  - `ExistingCandidateSelectorAdapter`
  - `ExistingGraphBuilderAdapter`
  - `ExistingRankerAdapter`
  - `ExistingPhraseBuilderAdapter`
  - `ExistingFormatterAdapter`
- Make BaseTextRank extractor call PipelineRunner internally.
- Add `StageReport` timings (cheap).
- Introduce `PipelineWorkspace` and thread it through runner (no output changes).

Goal: existing public APIs still work; performance unchanged (or within noise).

### Phase 2 — Personalization explicit (PositionRank, BiasedTextRank) + GIL release
- Add `TeleportBuilder` trait and `TeleportVector` artifact.
- Implement:
  - `UniformTeleportBuilder` (or `None`)
  - `PositionTeleportBuilder`
  - `FocusTermsTeleportBuilder`
- Switch PositionRank + BiasedTextRank pipelines to use the same Ranker with optional teleport.
- **Python**: release GIL during pipeline execution.
- Add Rayon runtime controls (`max_threads`, `single_thread`).

Testing:
- Unit tests for teleport normalization (sum to 1)
- Focus terms get higher probability
- Earlier terms get higher probability
- Integration tests verifying ordering stability

### Phase 3 — Graph construction modular (Base vs SingleRank)
Refactor GraphBuilder into smaller pieces:
- `WindowStrategy`: sentence-bounded vs cross-sentence
- `EdgeWeightPolicy`: binary vs count-accumulating

Implement:
- `WindowGraphBuilder { window_strategy, weight_policy }`

Map:
- BaseTextRank → sentence-bounded + binary
- SingleRank → cross-sentence + count-accumulating

Testing:
- Unit test edges for handcrafted token streams (across sentences)
- Confirm CSR construction determinism (especially in deterministic mode)

### Phase 4 — TopicalPageRank as “SingleRank + teleport”
- Add `TopicWeightsTeleportBuilder` reading:
  - `topic_weights` map and `topic_min_weight`
- Plug into existing PersonalizedPageRank engine.
- Ensure per-call overrides work as documented.

Testing:
- Normalize + min-weight behavior
- Integration test: changing weights changes top phrases

### Phase 5 — TopicRank/Multipartite “topic family” + GraphTransform stage
- Formalize clustering/graph transforms.
- Convert `src/clustering.rs` into a `Clusterer` stage.
- Add graph transforms as modules:
  - `IntraTopicEdgeRemover` (remove intra-cluster edges)
  - `AlphaBoostWeighter` (multipartite alpha boost)

Testing:
- Clustering determinism
- Intra-topic edge removal correctness
- Integration tests via JSON payloads

### Phase 6 — Result formatting as a first-class module + debug levels
Keep today’s output stable:
- phrases sorted by score desc (+ tie-breakers in deterministic mode)
- `converged`, `iterations`

Add debug enrichments (behind `debug` / `expose`):
- phrase spans/positions
- node scores (top K)
- cluster memberships
- convergence residuals/curve (bounded)
- graph stats (nodes/edges, window strategy)

Debug levels:
- `none` | `stats` | `top_nodes` | `full`
“full” may include bounded samples of adjacency and convergence curve.

### Phase 7 — Public modularity (power-user API)
- Rust: `PipelineBuilder` from PipelineSpec
- JSON: support `pipeline` object + presets
- Enforce precedence rules (modules override config override preset)

Add `validate_only` and capability discovery.

### Phase 8 — Batch + streaming execution (performance + usability)
- Rust: `PipelineRunner::run_batch(docs, cfg, observer)` with workspace reuse
- JSON: support JSONL/NDJSON streaming mode (avoid huge in-memory payloads)
- Python: generator-style batch API (avoid materializing full results list)

### Phase 9 (Optional, High-Impact) — SentenceRank / summarization pipeline family
Add a new pipeline family for sentence extraction (TextRank summarization style):
- `SentenceCandidates` (sentence units)
- `SentenceGraphBuilder` (similarity edges)
- Ranker (same)
- `SentenceFormatter`

Public API:
- `extract_sentences(text, top_n=...)`
- `SentenceRank` extractor class
- JSON: a `mode` or preset (e.g., `"preset": "sentence_rank"`)

Feature-flag initially to keep scope safe.

---

## 11) Testing Strategy (Make Modularity Pay Off)

### Unit tests per stage
- CandidateSelector:
  - include/exclude POS matches `include_pos`
  - `use_pos_in_nodes` changes node keys
  - stopword behavior matches rules (token flag or config list)
- GraphBuilder:
  - windowing correctness (within/cross sentence)
  - edge weights accumulate in SingleRank
  - CSR determinism in deterministic mode
- TeleportBuilder:
  - PositionRank: earlier tokens higher weight
  - BiasedTextRank: focus terms boosted
  - Topical: min-weight honored and normalized
- Ranker:
  - pagerank correctness on tiny graphs
  - convergence behavior and residual reporting
- PhraseBuilder:
  - min/max phrase length
  - aggregation / grouping correctness (lemma vs scrubbed_text)
- Validation:
  - missing stage errors
  - illegal combos errors
  - module_unavailable errors
  - limit_exceeded errors

### Integration / golden tests per variant
- stable sample texts from docs/examples
- assert:
  - phrase ordering stable (top-5 stable at minimum)
  - converged and iterations in expected range
  - deterministic-mode goldens match exactly

### Benchmarks and guardrails
- add CI thresholds to prevent perf regressions
- benchmark both default and deterministic modes
- benchmark batch mode (workspace reuse) separately

---

## 12) Documentation Changes

Update README/docs to make modularity explicit:

- “How TextRank Works” as a box diagram:
  Tokenization → Preprocess → Candidate selection → Graph build → (Transforms) → (Teleport) → Rank → Phrase build → Format

- “Variant = pipeline composition” table:
  show which stages differ for each variant

- “Advanced: inspect intermediates”:
  explain debug outputs and how to use them:
  - node scores (interpretability)
  - clusters (TopicRank/Multipartite)
  - convergence stats
  - stage timings

- “Production hardening”:
  strict mode, validate-only, runtime limits, deterministic mode, capability discovery

---

## 13) Migration and Compatibility

No breaking changes for:
- `extract_keywords()`
- extractor classes
- JSON `variant` strings and existing config fields

New features are optional:
- JSON `pipeline` object (optional)
- debug outputs (optional via `expose` or debug flag)
- deterministic mode (opt-in)
- runtime limits (optional defaults; safe sane defaults recommended)
- Rust `pipeline` public module behind a feature flag until stable

---

## Appendix A — Example PipelineSpec v1 (Explicit BaseTextRank)

```json
{
  "tokens": [ ... ],
  "config": { "language": "en", "top_n": 10 },
  "pipeline": {
    "v": 1,
    "modules": {
      "candidates": { "type": "word_nodes" },
      "graph": { "type": "cooccurrence_window", "window_size": 3, "cross_sentence": false, "edge_weighting": "binary" },
      "rank": { "type": "pagerank", "damping": 0.85, "max_iterations": 100, "convergence_threshold": 1e-6 },
      "phrases": { "type": "chunk_phrases", "min_phrase_length": 1, "max_phrase_length": 4, "score_aggregation": "sum", "phrase_grouping": "scrubbed_text" },
      "format": { "type": "standard_json" }
    },
    "runtime": { "deterministic": true },
    "expose": { "stage_timings": true }
  }
}
```

---

## Appendix B — Example: New Composition (SingleRank graph + Position teleport)

```json
{
  "tokens": [ ... ],
  "config": { "language": "en", "top_n": 10 },
  "pipeline": {
    "v": 1,
    "modules": {
      "candidates": { "type": "word_nodes" },
      "graph": { "type": "cooccurrence_window", "window_size": 3, "cross_sentence": true, "edge_weighting": "count" },
      "teleport": { "type": "position", "shape": "inverse_position" },
      "rank": { "type": "personalized_pagerank", "damping": 0.85, "max_iterations": 100, "convergence_threshold": 1e-6 },
      "phrases": { "type": "chunk_phrases", "min_phrase_length": 2, "max_phrase_length": 4, "phrase_grouping": "scrubbed_text" },
      "format": { "type": "standard_json_with_debug", "debug_key": "debug" }
    },
    "runtime": { "max_threads": 8 },
    "expose": { "node_scores": { "top_k": 25 }, "pagerank": { "residuals": true }, "stage_timings": true }
  }
}
```

---

## Appendix C — Rust type sketch (serde-friendly)

```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PipelineSpec {
    Preset(String),
    V1(PipelineSpecV1),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineSpecV1 {
    pub v: u32,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub modules: ModuleSet,
    #[serde(default)]
    pub expose: Option<ExposeSpec>,
    #[serde(default)]
    pub runtime: Option<RuntimeSpec>,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ModuleSet {
    pub preprocess: Option<PreprocessSpec>,
    pub candidates: Option<CandidatesSpec>,
    pub clustering: Option<ClusteringSpec>,
    pub graph: Option<GraphSpec>,
    pub teleport: Option<TeleportSpec>,
    pub rank: Option<RankSpec>,
    pub phrases: Option<PhraseSpec>,
    pub format: Option<FormatSpec>,
    #[serde(default)]
    pub graph_transforms: Vec<GraphTransformSpec>,
}
```
