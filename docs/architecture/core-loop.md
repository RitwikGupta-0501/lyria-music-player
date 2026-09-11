# Infinite Core Loop & Autoplay Architecture

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-13  
Source Modules: [`src-tauri/src/queue/autoplay.rs`](../../src-tauri/src/queue/autoplay.rs) │ [`src-tauri/src/db/queries.rs`](../../src-tauri/src/db/queries.rs#L1840-L2045) │ [`src/lib/stores/audio.svelte.ts`](../../src/lib/stores/audio.svelte.ts#L626-L662) │ [`src-tauri/src/providers/recommendations.rs`](../../src-tauri/src/providers/recommendations.rs#L413-L463) │ [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs#L723-L731)

---

# 1. Overview: The Subsystem Topology (The What)

The **Infinite Core Loop** is Lyria's automated playback replenishment engine. When an active playback queue reaches its terminal track (and repeat mode is disabled), the system dynamically synthesizes the next track and stages it into the playback pipeline for zero-latency gapless audio transitions.

> [!NOTE]
> **Orchestration Architecture:** Currently, replenishment is driven by the frontend audio store ([`src/lib/stores/audio.svelte.ts`](../../src/lib/stores/audio.svelte.ts)) invoking `resolve_autoplay_next_track`. The store stages the resolved candidate in `_stagedAutoplayTrack` and pipes it to the backend audio sink via `queue_next_audio`. When the audio thread transitions to the staged track, it emits `track-advanced`, prompting the store to commit the candidate into `QueueState` via `add_to_queue`. Migrating orchestration completely into the Rust audio daemon to uphold the "UI as pure remote control" directive is tracked under `ARCH-01`.

It implements a **Dual-Tier Waterfall Architecture**:
* **Tier 1 (Remote Federated WASM Radio):** Queries sandboxed WebAssembly extensions (e.g. YouTube, SoundCloud, Bandcamp) implementing the `get_radio` capability using a strict 3500ms timeout budget.
* **Tier 2 (Local Markov Affinity Random Walk):** Seamlessly falls back to local SQLite affinity vector scoring over the user's library with recency exclusions and Softmax temperature sampling.
* **Tier 3 (Terminal Boundary):** Returns `None` if both remote providers and local libraries are exhausted, causing the audio sink to drain cleanly and emit a `track-ended` event.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                          ACTIVE SEED TRACK                                       │
│                         seed: QueueTrack (Last Queued or Currently Playing)                      │
└──────────────────────────────────────────────────┬───────────────────────────────────────────────┘
                                                   │
                                    Autoplay Enabled & Queue at End?
                                                   │
                                  ┌────────────────┴────────────────┐
                                  │ YES                             │ NO
                                  ▼                                 ▼
┌──────────────────────────────────────────────────┐   ┌───────────────────────────────────────────┐
│     TIER 1: FEDERATED WASM RADIO STREAM          │   │         TERMINATE PLAYBACK CLEANLY        │
│   state.recommendation_compiler                  │   │      (Emit track-ended event to UI)       │
│   compile_federated_radio(&canonical_seed)       │   └───────────────────────────────────────────┘
│   - Enforces 3500ms execution timeout            │
│   - Excludes active seed canonical_key           │
│   - Falls back to default_remote_provider setting│
└─────────────────────────┬────────────────────────┘
                          │
          ┌───────────────┴───────────────┐
          │ SUCCESS                       │ TIMEOUT / EMPTY / OFFLINE
          ▼                               ▼
┌──────────────────────────┐   ┌───────────────────────────────────────────────────────────────────┐
│   RETURN REMOTE TRACK    │   │           TIER 2: LOCAL MARKOV RANDOM WALK (queries.rs)           │
│  TrackSourceInfo::Remote │   │   DbRequest::GetMarkovAutoplayCandidate                           │
│  - Stream URL & Headers  │   │   - 120-Minute Recency Exclusion Filter (anti-repetition)         │
│  - Lazy resolution       │   │   - Multi-Attribute Affinity Vector Scoring (Weights: 40/20/15/10)│
│  - Staged to audio sink  │   │   - Softmax Temperature Sampling over Top-15 Pool (T = 14.0)      │
│  - Appended on advance   │   └─────────────────────────────────┬─────────────────────────────────┘
└──────────────────────────┘                                     │
                                                 ┌───────────────┴───────────────┐
                                                 │ CANDIDATE FOUND               │ EMPTY LIBRARY
                                                 ▼                               ▼
                               ┌──────────────────────────────────┐   ┌────────────────────────────┐
                               │        RETURN LOCAL TRACK        │   │    STOP PLAYBACK CLEANLY   │
                               │      TrackSourceInfo::Local      │   │  (Zero tracks in library)  │
                               │      - Staged to audio sink      │   └────────────────────────────┘
                               │      - Appended on advance       │
                               └──────────────────────────────────┘
```

---

# 2. Motivation & Design Goals (The Why)

Algorithmic radio engines in commercial streaming services suffer from two major pathologies:

### 2.1. The "Echo Chamber" Loop Trap
Naive greedy recommendation algorithms prioritize songs with the highest immediate affinity score. This results in **deterministic loop traps**, where playing an artist like *Radiohead* repeatedly queues the same 3 top-streamed songs (*Creep*, *Karma Police*, *No Surprises*), cycling indefinitely in an echo chamber.

Lyria overcomes this through **three anti-loop safeguards**:
1. **120-Minute Strict Recency Exclusion:** Tracks played within the last 2 hours are excluded from the candidate pool.
2. **24-Hour Continuous Decay Penalty:** Tracks played within the last 24 hours receive an hourly decaying penalty.
3. **Softmax Temperature Sampling:** Candidates are chosen probabilistically rather than greedily, granting lower-ranked high-affinity tracks an organic opportunity to play.

### 2.2. Offline Resilience & Privacy
Most modern players require active cloud connectivity for radio functionality. If internet access drops, autoplay dies.

Lyria guarantees continuous playback by falling back to an **offline local Markov walk** directly over the user's SQLite database. If remote WASM providers are offline, rate-limited, or timing out, the user never hears a break in music.

---

# 3. Architectural Decisions & Trade-Offs (The Reasoning)

### 3.1. Dual-Tier Waterfall vs. Single Unified Model
* **Decision:** Remote WASM extensions are queried first with a 3500ms timeout budget; if unfulfilled, the engine falls back to local SQLite heuristics.
* **Reasoning:** Remote providers possess vast global discovery graphs with genre-matched radio streams. However, external APIs introduce network latency and failure risk. A waterfall architecture leverages rich external graphs when available while guaranteeing zero dropouts via local fallback.
* **Trade-Off:** Remote radio queries must complete within 3500ms before the current track finishes to ensure gapless transition staging.

### 3.2. Softmax Temperature Sampling vs. Greedy Argmax
* **Decision:** The local Markov engine truncates candidates to the Top-15 by raw score and samples probabilistically using a Boltzmann distribution with temperature $T = 14.0$.
* **Reasoning:** 
  * Greedy selection ($\text{argmax}$) produces identical, predictable sequences every time a seed track is played.
  * Pure uniform random sampling produces jarring genre/mood whiplash (e.g. classical acoustic following heavy techno).
  * Softmax temperature sampling balances **exploitation** (strongly favoring high-affinity tracks) with **exploration** (allowing related tracks to surface).
* **Trade-Off:** Requires computing exponential weights and cumulative distribution roulette-wheel sampling in Rust, adding negligible CPU overhead ($<0.5\text{ms}$) per track transition.

---

# 4. Mathematical Model & Scoring Formulation (The How)

The local engine ([`src-tauri/src/db/queries.rs:1840-2045`](../../src-tauri/src/db/queries.rs#L1840-L2045)) executes a multi-stage probabilistic pipeline:

### 4.1. Stage 1: Candidate Pool Filtering
From all indexed local tracks, the engine filters candidates:
1. **Seed Exclusion:** Excludes the current seed track (`cand.id != seed.id` and `cand.file_path != seed.file_path`).
2. **120-Minute Recency Gate:** Excludes tracks where:
   $$\text{last\_played\_at} \ge \text{datetime}('now', '-120 minutes')$$
3. **Library Size Safety Fallback:** If the remaining candidate pool contains $< 3$ tracks (common in small or fresh libraries), the 120-minute filter is relaxed to prevent silence.

---

### 4.2. Stage 2: Multi-Factor Affinity Vector Scoring
Each candidate track $c$ is evaluated against seed $s$ across 8 orthogonal dimensions:

$$\text{RawScore}(c) = S_{\text{artist}} + S_{\text{album}} + S_{\text{lexical}} + S_{\text{duration}} + B_{\text{liked}} + B_{\text{plays}} - P_{\text{recency}} + \mathcal{U}(0, 5)$$

$$\text{FinalScore}(c) = \max(0.1, \text{RawScore}(c))$$

| Factor | Weight Range | Scoring Logic & Code Implementation |
| :--- | :---: | :--- |
| **Artist Affinity ($S_{\text{artist}}$)** | $0.0 - 40.0$ | Exact artist match = $+40.0$. Substring or collaboration match = $+20.0$. |
| **Album Affinity ($S_{\text{album}}$)** | $0.0 - 20.0$ | Candidate belongs to identical album ID = $+20.0$. |
| **Lexical Affinity ($S_{\text{lexical}}$)** | $0.0 - 15.0$ | Matches primary token of artist name (genre/sub-label proxy) = $+15.0$. |
| **Duration Proximity ($S_{\text{duration}}$)** | $0.0 - 10.0$ | Proximity within a 3-minute window: $\Delta d = |d_s - d_c|$; Score $= 10.0 \cdot \left(1.0 - \frac{\Delta d}{180\,000\text{ms}}\right)$. Fallback $= 5.0$.<br>*(Note: Local seeds currently lack duration metadata in `autoplay.rs` and default to the 5.0 fallback; tracked under `BUG-05`)* |
| **Liked Bonus ($B_{\text{liked}}$)** | $+15.0$ | Favor user favorites: $+15.0$ if candidate track is liked. |
| **Play Count Bonus ($B_{\text{plays}}$)** | $0.0 - 15.0$ | Logarithmic popularity scaling: $\min(15.0, \ln(\text{play\_count} + 1) \cdot 3.75)$. |
| **Recency Decay Penalty ($P_{\text{recency}}$)** | $0.0 - 20.0$ | If played within last 24h: Penalty $= 20.0 \cdot \left(1.0 - \frac{\text{hours\_since\_played}}{24.0}\right)$. |
| **Stochastic Exploration Noise** | $0.0 - 5.0$ | Uniform random jitter $\mathcal{U}(0, 5)$ injected per candidate to break mathematical ties. |

---

### 4.3. Stage 3: Top-K Truncation & Softmax Boltzmann Sampling

1. **Top-K Truncation:** Sort all candidates descending by $\text{FinalScore}$ and retain the top $K = 15$ candidates.
2. **Numerically Stable Boltzmann Weighting:**  
   To avoid floating-point overflow during exponentiation, subtract the maximum score $s_{\max}$:
   $$w_i = \exp\left(\frac{s_i - s_{\max}}{T}\right)$$
   Where temperature $T = 0.7 \times 20.0 = 14.0$.
3. **Probability Distribution:**
   $$P(i) = \frac{w_i}{\sum_{j=1}^{15} w_j}$$
4. **Roulette-Wheel Sampling:** Draw sample target $r \sim \mathcal{U}(0, \sum w)$, accumulating weights until $r \le \sum_{k=1}^{i} w_k$. Return candidate $i$.

---

# 5. Remote Federated Radio Pipeline (`autoplay.rs`)

When third-party WASM extensions are loaded:

1. **Seed Canonicalization:** Packages current playback into [`CanonicalSeedV1`](../../src-tauri/src/providers/mod.rs):
   ```rust
   let canonical_seed = CanonicalSeedV1 {
       abi_version: 1,
       canonical_key: format!("{}::{}", artist.to_lowercase(), title.to_lowercase()),
       title,
       artist,
       album: None,
       isrc: None,
       duration_ms,
       native_id,
       provider_id,
   };
   ```
2. **Timeout Execution:** Dispatches `compile_federated_radio(&canonical_seed)` wrapped in `tokio::time::timeout(Duration::from_millis(3500), ...)`.
3. **Deduplication & Provider Resolution:** Filters out any candidate whose normalized `artist::title` matches the active seed. If a candidate lacks an `isrc` provider ID, queries SQLite setting `default_remote_provider` (falling back to `"youtube-wasm"`).
4. **Stage & Queue Commitment:** If valid tracks are returned, instantiates `QueueTrack` with a UUID `instance_id`. The frontend stages it into the `rodio` sink via `queue_next_audio` for zero-latency gapless transition, and commits it into `QueueState` via `add_to_queue` once the audio thread advances.

---

# 6. Verification & Automated Quality Gates

The Infinite Core Loop and recommendation pipelines are validated through automated test coverage and resource benchmarks:

```bash
# 1. Run Database Telemetry & Cold Start Unit Tests (Markov unit tests tracked under IMPR-05)
cargo test db::queries::tests

# 2. Run Queue Subsystem Unit Tests
cargo test queue::tests

# 3. Memory & Drift Endurance Benchmark (Passively monitors heap drift slope <= 0.5 KB/min)
./scripts/benchmark-memory.sh --long-soak --duration 1800

# 4. Rapid Lifecycle & Leak Gate Benchmark (Evaluates leak gate <= 5.0MB & thread cleanup <= 0)
./scripts/benchmark-memory.sh --full-suite
```
