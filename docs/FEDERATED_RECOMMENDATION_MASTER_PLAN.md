# Final Hardened Master Implementation Plan: Extension-Agnostic Federated Recommendation Engine (v4)

**Goal:** Build a resilient, privacy-first, extension-agnostic recommendation system for Echo Desktop that synthesizes local playback telemetry (`rusqlite`) with concurrent sandboxed WebAssembly provider extensions (`Extism`) into a single, unified, deduplicated discovery feed.

---

## 1. System Architecture: The Hardened Federated Pipeline

```mermaid
flowchart TD
    subgraph Local_Storage ["1. Local Telemetry & Seed Engine (rusqlite WAL)"]
        A[User Plays Song] --> B[Log to playback_events & song_telemetry]
        B --> C[Compute Top Seeds: Most Played / Liked / Recent]
        C --> D[Generate CanonicalSeedV1: Canonical Key + NFKC + ISRC + Duration]
    end

    subgraph Cache_Layer ["2. Canonical Per-Seed Cache & Invalidation (SQLite)"]
        D --> E{Check (seed.canonical_key, provider_id, shelf_type) in recommendation_cache}
        E -- Cache Hits --> L1[Collect Cached Recommendations]
        E -- Cache Misses --> F[Collect Missing Seed-Provider Pairs]
    end

    subgraph WASM_Dispatch ["3. Bound Concurrency, Cancellation & Circuit Breakers"]
        F --> G[AppState::in_flight_cancel Swap & Abort Prior Queries]
        G --> H[FuturesUnordered with Per-Call 4s Timeout + CancellationToken]
        H --> I1[Extension A: YouTube WASM]
        H --> I2[Extension B: Spotify / SoundCloud WASM]
        H --> I3[Circuit Breaker Check: Skip providers in backoff cooldown]
        H -->|Timeout / Panic / 429| I4[record_provider_failure: Differentiated Backoff & Eviction]
    end

    subgraph Compiler_Dedup ["4. Multi-Source Compilation & Confidence Matching"]
        I1 & I2 & L1 --> J[Unified Normalizer: NFKC + Token Cleaning]
        J --> K{ISRC Available?}
        K -- Yes: Exact Match --> M1[100% Confidence Merge]
        K -- No --> M2[Continuous Gaussian Scoring: Token-Set + Artist + Duration]
        M2 --> M3{Score >= 0.85 & Lexicographical Overrides Checked}
        M3 -- Yes --> M1
        M3 -- No --> M4[Keep as Distinct Tracks]
        M1 & M4 --> ReRank[Re-ranking: Provider Priority + Telemetry + Health Metric]
        ReRank --> SaveCache[Write Individual Missing Seeds to recommendation_cache using seed.canonical_key]
    end

    SaveCache --> Out[Two-Phase SWR Stream]

    subgraph Svelte5_Frontend ["5. Svelte 5 SWR Store & UI"]
        Out --> N1[Phase 1: Instant Local Read <10ms via Direct Connection]
        Out --> N2[Phase 2: Async Streaming Infill with Request Generation Counter]
        N1 & N2 --> O[Home View: Quick Picks, Daily Discover, Similar To, Failure Badges]
    end
```

---

## 2. Hard Invariants & Core Structural Fixes

> [!IMPORTANT]
> **1. Structural Host-Function Capability Separation (No Content Sniffing):**
> - The host does **not** pattern-match HTTP headers or payload content.
> - Instead, the host exposes **two structurally distinct host functions**:
>   1. `host_http_request`: Standard network call for searching, browse modules, and resolving streams.
>   2. `host_telemetry_request`: Dedicated telemetry dispatch function.
> - **Enforcement Invariant:** When constructing an `Extism::Plugin`, `host_telemetry_request` is **structurally withheld or hard-rejected** with `PermissionDenied` unless the extension's manifest declares `"telemetry_reporting"`. Non-compliant extensions cannot invoke telemetry regardless of payload shape.
> - `notify_playback_completed` calls an extension **strictly for its own originating stream** and passes zero local file paths or foreign extension IDs.

> [!IMPORTANT]
> **2. True Canonical Keying for Per-Seed Cache:**
> - `CanonicalSeedV1` explicitly holds `pub canonical_key: String`, computed at construction time via `canonical::make_canonical_key(&title, &artist)` (incorporating Unicode NFKC, lowercasing, diacritic stripping, and noise removal).
> - `recommendation_cache` is indexed and queried strictly via `seed.canonical_key`:  
>   `PRIMARY KEY (seed_canonical_key, provider_id, shelf_type)`
> - Example: `"Café"` and `"Cafe"` both resolve to `artist::cafe`, hitting the **exact same cache entry** and preventing cache fragmentation.

> [!IMPORTANT]
> **3. AppState-Managed In-Flight Request Cancellation:**
> - `AppState` holds `in_flight_recommendation_cancel: Arc<Mutex<Option<CancellationToken>>>`.
> - When `get_home_remote_shelves` is invoked from Tauri, it atomically swaps the held token, calling `old_token.cancel()`. Prior in-flight WASM futures abort immediately, freeing `Semaphore(4)` permits.

> [!IMPORTANT]
> **4. Typed Circuit Breaker with Differentiated Backoff:**
> - `record_provider_failure(provider_id, failure_type)` handles specific errors:
>   - `Http429` (Rate-Limited) $\implies$ Immediate 120s cooldown.
>   - `Timeout` (4s exceeded) $\implies$ Increment consecutive failures; after 3 failures $\implies$ 60s cooldown.
>   - `WasmPanic` $\implies$ Evict plugin from cache, re-instantiate, and enter 60s cooldown.
>   - `AuthExpired` $\implies$ Mark provider degraded and notify UI.

> [!IMPORTANT]
> **5. Continuous Gaussian Duration Penalty & Order-Invariant Overrides:**
> - Gaussian duration penalty: $\text{penalty} = \exp\left(-\frac{1}{2} \left(\frac{|\Delta t_{\text{ms}}|}{10000}\right)^2\right)$ (zero cliff discontinuities).
> - All `dedup_overrides` lookups sort keys lexicographically:  
>   `let (k1, k2) = if key_a <= key_b { (key_a, key_b) } else { (key_b, key_a) };`

---

## 3. Phase-Wise Implementation Roadmap

```
Final Hardened Master Plan: Extension-Agnostic Recommendation Engine (v4)
 ├── Phase 1: Structural Host-Function Capabilities, ABI v1 & Extension Health Manager
 ├── Phase 2: Unicode NFKC Normalization, ISRC & Gaussian Confidence Deduplication
 ├── Phase 3: RecommendationCompiler, Canonical Cache, Cancellation & Circuit Breakers
 ├── Phase 4: Svelte 5 Two-Phase Store with Generation Counters & Error Boundaries
 └── Phase 5: Adversarial Test Harness, Mock Plugins & Resilient Home UI
```

---

## 4. Detailed Component Specifications

### Phase 1: Structural Host-Function Capabilities, ABI v1 & Extension Health Manager

#### [MODIFY] `src-tauri/src/providers/mod.rs`
- **Versioned ABI Structs & Manifest:**
  ```rust
  pub const PROVIDER_ABI_VERSION: u32 = 1;

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct CanonicalSeedV1 {
      pub abi_version: u32,
      pub canonical_key: String, // Normalized via canonical::make_canonical_key()
      pub title: String,
      pub artist: String,
      pub album: Option<String>,
      pub isrc: Option<String>,
      pub duration_ms: Option<u64>,
      pub native_id: Option<String>,
      pub provider_id: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct RadioStreamResultV1 {
      pub tracks: Vec<TrackResult>,
      pub continuation_token: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PlaybackTelemetryEventV1 {
      pub native_track_id: String,
      pub duration_ms: u64,
      pub total_track_duration_ms: u64,
      pub completed: bool,
  }
  ```
- **Structural Host-Function Separation in `wasm_bridge.rs`:**
  - `host_http_request`: Standard network access for core playback and discovery queries.
  - `host_telemetry_request`: Dedicated telemetry dispatch function. When registering functions in `extism::PluginBuilder`:
    ```rust
    let builder = extism::PluginBuilder::new(manifest)
        .with_function("host_http_request", [extism::ValType::I64], [extism::ValType::I64], user_data_reqwest, host_http_request);
    
    // Gated structurally on manifest capability
    if provider.capabilities.contains(&"telemetry_reporting".to_string()) {
        builder = builder.with_function("host_telemetry_request", [extism::ValType::I64], [extism::ValType::I64], user_data_telemetry, host_telemetry_request);
    } else {
        builder = builder.with_function("host_telemetry_request", [extism::ValType::I64], [extism::ValType::I64], user_data_none, host_telemetry_blocked);
    }
    ```
- **Extension Health Manager:**
  - Evict poisoned plugin instances on panic/timeout and recreate cleanly on next request.
- **Strict Concurrency & Timeouts:**
  - Global WASM semaphore: `Semaphore::new(4)`.
  - Per-provider concurrency limit: `Semaphore::new(2)`.
  - Enforce per-call strict timeout using `tokio::time::timeout(Duration::from_secs(4), ...)`.

#### [MODIFY] `extensions/youtube-wasm/src/lib.rs`
- Export ABI v1 functions: `get_related`, `get_radio`, `on_playback_event`.
- `on_playback_event` calls `host_telemetry_request` to submit `registerPlayback` pings against `videostatsPlaybackUrl`.

---

### Phase 2: Unicode NFKC Normalization, ISRC & Gaussian Confidence Deduplication

#### [MODIFY] `src-tauri/src/db/canonical.rs`
- **Unicode NFKC & Diacritic Normalization (English-Priority):**
  - Convert strings using `unicode_normalization::UnicodeNormalization::nfkc()`.
  - Strip accents/diacritics for cross-script matching while preserving original alphanumeric tokens.
- **Continuous Gaussian Confidence Scoring & Order-Invariant Overrides:**
  ```rust
  pub struct DedupMatchResult {
      pub is_match: bool,
      pub confidence: f32,
      pub reason: &'static str,
  }

  pub fn canonical_override_key(key_a: &str, key_b: &str) -> (String, String) {
      if key_a <= key_b {
          (key_a.to_string(), key_b.to_string())
      } else {
          (key_b.to_string(), key_a.to_string())
      }
  }

  pub fn compute_dedup_confidence(
      track_a: &CandidateTrack,
      track_b: &CandidateTrack,
      overrides: &HashMap<(String, String), bool>,
  ) -> DedupMatchResult {
      // 0. Check Order-Invariant User Overrides
      let pair = canonical_override_key(&track_a.canonical_key, &track_b.canonical_key);
      if let Some(&forced) = overrides.get(&pair) {
          return DedupMatchResult { is_match: forced, confidence: if forced { 1.0 } else { 0.0 }, reason: "user_override" };
      }
      // 1. ISRC exact match short-circuit
      if let (Some(isrc_a), Some(isrc_b)) = (&track_a.isrc, &track_b.isrc) {
          if !isrc_a.is_empty() && isrc_a == isrc_b {
              return DedupMatchResult { is_match: true, confidence: 1.0, reason: "isrc_match" };
          }
      }
      // 2. Continuous Gaussian Scoring
      let title_sim = strsim::sorensen_dice(&clean_title(&track_a.title), &clean_title(&track_b.title)) as f32;
      let artist_sim = strsim::sorensen_dice(&clean_artist(&track_a.artist), &clean_artist(&track_b.artist)) as f32;
      
      let duration_penalty = match (track_a.duration_ms, track_b.duration_ms) {
          (Some(d_a), Some(d_b)) => {
              let diff_sec = (d_a as f32 - d_b as f32).abs() / 1000.0;
              (-0.5 * (diff_sec / 10.0).powi(2)).exp()
          }
          _ => 0.85, // Neutral penalty for missing metadata
      };

      let confidence = (title_sim * 0.50) + (artist_sim * 0.30) + (duration_penalty * 0.20);
      DedupMatchResult {
          is_match: confidence >= 0.85,
          confidence,
          reason: "fuzzy_heuristic",
      }
  }
  ```

---

### Phase 3: RecommendationCompiler, Canonical Cache, Cancellation & Circuit Breakers

#### [NEW] `src-tauri/src/db/schema.rs: Migrations`
- Add canonical per-seed cache and circuit breaker tracking:
  ```sql
  CREATE TABLE IF NOT EXISTS recommendation_cache (
      seed_canonical_key TEXT NOT NULL,
      provider_id TEXT NOT NULL,
      shelf_type TEXT NOT NULL,
      payload_json TEXT NOT NULL,
      fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
      ttl_seconds INTEGER NOT NULL DEFAULT 21600,
      PRIMARY KEY(seed_canonical_key, provider_id, shelf_type)
  );
  CREATE INDEX IF NOT EXISTS idx_rec_cache_lookup 
      ON recommendation_cache(seed_canonical_key, provider_id, shelf_type, fetched_at);

  CREATE TABLE IF NOT EXISTS dedup_overrides (
      canonical_key_a TEXT NOT NULL,
      canonical_key_b TEXT NOT NULL,
      should_merge INTEGER NOT NULL,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
      PRIMARY KEY(canonical_key_a, canonical_key_b)
  );

  ALTER TABLE extension_metrics ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0;
  ALTER TABLE extension_metrics ADD COLUMN backoff_until TIMESTAMP;
  ```

#### [NEW] `src-tauri/src/providers/recommendations.rs`
- **Canonical Cache Lookup, Circuit Breakers & In-Flight Cancellation:**
  ```rust
  #[derive(Debug)]
  pub enum ProviderErrorType {
      Timeout,
      Http429,
      AuthExpired,
      WasmPanic(String),
      Generic(String),
  }

  pub struct FederatedShelfResult {
      pub tracks: Vec<FederatedTrack>,
      pub failed_providers: Vec<String>,
      pub is_partial: bool,
  }

  pub async fn get_federated_daily_discover(
      &self,
      seeds: Vec<CanonicalSeedV1>,
      cancel_token: CancellationToken,
  ) -> FederatedShelfResult {
      let mut cached_tracks = Vec::new();
      let mut missing_queries = Vec::new();

      // 1. Check Canonical Per-Seed Cache
      for seed in &seeds {
          for provider in self.provider_manager.get_active_providers_with_cap("related") {
              if let Some(cached) = self.get_valid_cache(&seed.canonical_key, &provider.id, "daily_discover") {
                  cached_tracks.extend(cached);
              } else {
                  missing_queries.push((provider.id.clone(), seed.clone()));
              }
          }
      }

      // 2. Fan-out only for missing seed-provider pairs with Circuit Breaker
      let mut tasks = FuturesUnordered::new();
      for (p_id, seed) in missing_queries {
          if self.is_provider_in_backoff(&p_id) {
              continue;
          }
          let p_mgr = self.provider_manager.clone();
          let c_token = cancel_token.clone();
          tasks.push(async move {
              tokio::select! {
                  _ = c_token.cancelled() => Err((p_id, seed, ProviderErrorType::Generic("cancelled".into()))),
                  res = tokio::time::timeout(Duration::from_secs(4), async {
                      p_mgr.get_related_for_seed(&p_id, &seed).await
                  }) => match res {
                      Ok(Ok(tracks)) => Ok((p_id, seed, tracks)),
                      Ok(Err(e)) => Err((p_id, seed, ProviderErrorType::Generic(e.to_string()))),
                      Err(_) => Err((p_id, seed, ProviderErrorType::Timeout)),
                  }
              }
          });
      }

      let mut collected_tracks = cached_tracks.clone();
      let mut failed_providers = Vec::new();

      while let Some(res) = tasks.next().await {
          match res {
              Ok((p_id, seed, tracks)) => {
                  self.record_provider_success(&p_id);
                  self.save_cache(&seed.canonical_key, &p_id, "daily_discover", &tracks);
                  collected_tracks.extend(tracks);
              }
              Err((_p_id, _seed, ProviderErrorType::Generic(msg))) if msg == "cancelled" => {
                  // If superseded, return whatever cache hits were already collected
                  return FederatedShelfResult { tracks: self.compiler.compile_and_deduplicate(cached_tracks), failed_providers: vec!["cancelled".into()], is_partial: true };
              }
              Err((p_id, _seed, err_type)) => {
                  self.record_provider_failure(&p_id, &err_type);
                  failed_providers.push(format!("{}: {:?}", p_id, err_type));
              }
          }
      }

      let merged = self.compiler.compile_and_deduplicate(collected_tracks);
      FederatedShelfResult {
          tracks: merged,
          failed_providers,
          is_partial: !failed_providers.is_empty(),
      }
  }
  ```

#### [MODIFY] `src-tauri/src/lib.rs: AppState & Tauri Commands`
- Wire cancellation token swap-and-abort in `AppState`:
  ```rust
  pub struct AppState {
      pub in_flight_recommendation_cancel: Arc<std::sync::Mutex<Option<CancellationToken>>>,
      // ...
  }

  #[tauri::command]
  pub async fn get_home_remote_shelves(
      state: tauri::State<'_, AppState>,
  ) -> Result<FederatedShelfResult, String> {
      let new_token = CancellationToken::new();
      {
          let mut lock = state.in_flight_recommendation_cancel.lock().unwrap();
          if let Some(old_token) = lock.take() {
              old_token.cancel(); // Abort prior in-flight fan-out immediately
          }
          *lock = Some(new_token.clone());
      }
      
      let seeds = state.recommendation_compiler.get_daily_discover_seeds().await?;
      Ok(state.recommendation_compiler.get_federated_daily_discover(seeds, new_token).await)
  }
  ```

---

### Phase 4: Svelte 5 Two-Phase Store with Generation Counters & Error Boundaries

#### [NEW] `src/lib/stores/homeStore.svelte.ts`
- **Generation-Guarded Asynchronous SWR Store:**
  ```typescript
  class HomeStore {
      quickPicks = $state<FederatedTrack[]>([]);
      keepListening = $state<FederatedTrack[]>([]);
      dailyDiscover = $state<FederatedTrack[]>([]);
      similarShelves = $state<SimilarShelf[]>([]);
      
      isLoadingLocal = $state(false);
      isLoadingRemote = $state(false);
      errorLocal = $state<string | null>(null);
      errorRemote = $state<string | null>(null);
      failedProviders = $state<string[]>([]);

      private currentFetchId = 0;

      async load() {
          const fetchId = ++this.currentFetchId;

          // Phase 1: Local Shelves (<10ms)
          this.isLoadingLocal = true;
          this.errorLocal = null;
          try {
              const local = await invoke<HomeLocalShelves>('get_home_local_shelves');
              if (fetchId !== this.currentFetchId) return; // Discard stale response
              this.quickPicks = local.quickPicks;
              this.keepListening = local.keepListening;
          } catch (e: any) {
              if (fetchId === this.currentFetchId) this.errorLocal = e?.toString() || 'Failed to load local history';
          } finally {
              if (fetchId === this.currentFetchId) this.isLoadingLocal = false;
          }

          // Phase 2: Remote Federated Discovery (Backend cancels prior in-flight queries)
          this.isLoadingRemote = true;
          this.errorRemote = null;
          try {
              const remote = await invoke<FederatedShelfResult>('get_home_remote_shelves');
              if (fetchId !== this.currentFetchId) return; // Discard stale response
              this.dailyDiscover = remote.tracks;
              this.failedProviders = remote.failed_providers;
          } catch (e: any) {
              if (fetchId === this.currentFetchId) this.errorRemote = e?.toString() || 'Failed to fetch recommendations';
          } finally {
              if (fetchId === this.currentFetchId) this.isLoadingRemote = false;
          }
      }
  }
  ```

---

### Phase 5: Adversarial Test Harness & Resilient UI

#### [NEW] `src-tauri/src/providers/tests/adversarial_tests.rs`
- **Adversarial Extension Mock Suite:**
  1. `test_slow_hanging_extension_timeout`: Simulates an extension sleeping 10s; verifies 4s cutoff and partial result return.
  2. `test_crashing_extension_isolation`: Simulates a panic in WASM; verifies plugin is evicted from memory and other extensions complete.
  3. `test_circuit_breaker_typed_backoff`: Fails 3 consecutive calls; verifies 4th call skips WASM invocation and logs backoff.
  4. `test_order_invariant_user_override`: Asserts `(a, b)` and `(b, a)` both trigger manual dedup override correctly.
  5. `test_canonical_key_cache_collision`: Asserts `"Café"` and `"Cafe"` map to the same cache entry and hit the cache without re-fanout.
  6. `test_host_telemetry_structural_rejection`: Asserts that an extension lacking `"telemetry_reporting"` is rejected at the `host_telemetry_request` boundary regardless of request payload content.
  7. `test_in_flight_cancellation_permits`: Asserts that triggering a new search immediately cancels in-flight futures and returns semaphore permits to 4.

---

## 5. Verification Plan

### Automated Test Suite
```bash
# 1. Run canonical Gaussian scoring & order-invariant override unit tests
cargo test --package echo-desktop --lib db::canonical::tests

# 2. Run adversarial multi-provider failure & circuit breaker tests
cargo test --package echo-desktop --lib providers::tests::adversarial_tests

# 3. Frontend type checking and rune validation
bun run check

# 4. Backend strict linting
cargo clippy --all-targets -- -D warnings
```

### Manual Verification Checklist
1. **Host Network Guard Check:** Inspect outbound network traffic during playback; verify telemetry events are dispatched only via `host_telemetry_request` to the active stream's provider if enabled.
2. **Partial Cache Hit Check:** Change 1 seed song; verify only the new seed triggers network calls while the other 4 load instantly from SQLite cache.
3. **Circuit Breaker Check:** Break one extension's URL; verify that after 3 failures it enters cooldown and Home screen loads in $<10\text{ms}$ with a failure badge.
4. **Rapid Tab Navigation Test:** Switch tabs 10 times quickly; verify in-flight tasks cancel cleanly and terminal shows zero orphaned requests or semaphore lockups.
