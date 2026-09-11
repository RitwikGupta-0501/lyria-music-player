# Queue State Machine & Playback Invariants

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-13  
Source Module: [`src-tauri/src/queue/`](../../src-tauri/src/queue/)

---

# 1. Overview: The Subsystem Topology (The What)

The Lyria Queue System is an in-memory, thread-safe state machine governing track sequencing, shuffle permutation, repeat behaviors, and automated queue replenishment.

It serves as the single source of truth for playback order, holding both local filesystem files and remote WebAssembly streaming tracks in a unified vector while persisting snapshots to SQLite for cross-session recovery.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   SVELTE 5 FRONTEND (Remote Control)                             │
│       PlayerBar.svelte  │  QueueSidebar.svelte  │  TrackRow.svelte  │  GlobalSearch.svelte       │
└───────────────────────────────────┬──────────────────────────────────┬───────────────────────────┘
                                    │                                  │
                  invoke("add_to_queue", ...)                          │ listen("queue-changed")
                  invoke("skip_forward", ...)                          │ (QueueChangeEvent)
                  invoke("set_shuffle", ...)                           │
                  invoke("resolve_autoplay_next_track", ...)           │
                                    ▼                                  │
┌──────────────────────────────────────────────────────────────────────┴───────────────────────────┐
│                               TAURI QUEUE COMMAND BRIDGE (commands.rs)                           │
│                     AppState.queue: std::sync::Mutex<QueueState>                                 │
└───────────────────────────────────┬──────────────────────────────────┬───────────────────────────┘
                                    │                                  │
          Atomic Mutation           ▼                                  ▼  Autoplay Resolver
┌──────────────────────────────────────────────┐                ┌──────────────────────────────────┐
│              QueueState STRUCT               │                │      autoplay.rs RESOLVER        │
│  - tracks: Vec<QueueTrack>                   │                │  1. Federated WASM Radio Stream  │
│  - current_position: usize                   │                │  2. Local Markov Random Walk     │
│  - repeat_mode: RepeatMode (Off/All/One)     │                │  3. Boundary Exhaustion (None)   │
│  - mode: QueueMode (Normal/Shuffle)          │                └─────────────────┬────────────────┘
│  - shuffle_state: Option<ShuffleState>       │                                  │
└───────────────────┬──────────────────────────┘                                  │
                    │                                                             │
                    │ SQLite Schema & Recovery (recovery.rs)                      ▼
                    ▼                                              Resolved Candidate Track
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 SQLITE QUEUE PERSISTENCE TABLES                                  │
│       queue_state  │  queued_tracks  │  shuffle_order  │  queue_history  │  queue_snapshots      │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Core Responsibilities
1. **Atomic State Isolation:** All queue mutations occur within an atomic `std::sync::Mutex<QueueState>` lock, ensuring zero race conditions between user UI clicks, audio thread EOF notifications, and background autoplay additions.
2. **Non-Destructive Shuffle:** Shuffling randomizes playback order without altering the original queue order. Toggling shuffle off seamlessly restores the user's initial sequence while keeping the active song playing.
3. **Session Recovery Architecture:** Schema and recovery logic (`persistence.rs`, `recovery.rs`) support restoring queue states across restarts. (Active snapshotting on every queue mutation is tracked in `TODO.md` under FEAT-03).
4. **Dual-Tier Autoplay Replenishment:** When a non-repeating queue reaches its terminus, `autoplay.rs` queries federated WASM radio streams with a seamless fallback to local Markov affinity random walks. (Autoplay is currently coordinated via the frontend store; daemon migration is tracked in `TODO.md` under ARCH-01).
5. **Deterministic Event Broadcasting:** State changes emit a `queue-changed` event (`QueueChangeEvent`) containing the complete serialized queue, active position, and mode flags to keep all UI windows in sync.

---

# 2. Motivation & Engineering Invariants (The Why)

Desktop music queues frequently exhibit subtle, frustrating edge-case bugs:

### 2.1. The Destructive Shuffle Flaw
Many legacy media players implement shuffle by physically sorting the underlying track list in-place (`tracks.shuffle()`). When the listener turns shuffle off, their original album order or custom playlist sequence is permanently destroyed.

Lyria treats the underlying track list as an **immutable sequential array**. Shuffle is modeled as an auxiliary permutation vector (`ShuffleState`) referencing track instance UUIDs.

### 2.2. Duplicate Track Disambiguation
Users frequently add the same song to a queue multiple times (e.g. track 1 and track 15). If the queue is keyed by file path or database ID, shuffle algorithms and reordering commands become ambiguous, causing incorrect cursor jumps.

Lyria assigns a **unique session UUID (`instance_id`)** to every queued item upon insertion. Identical tracks maintain distinct identities, allowing precise jump, drag-and-drop reordering, and shuffle indexing.

---

# 3. Key Architectural Decisions & Trade-Offs (The Reasoning)

### 3.1. Unified `TrackSourceInfo` Representation
* **Decision:** Queue items store source information as an explicit enum:
  ```rust
  pub enum TrackSourceInfo {
      Local {
          track_id: i64,
          file_path: String,
          album_id: Option<i64>,
      },
      Remote {
          provider_id: String,
          remote_track_id: String,
          stream_url: Option<String>,
          quality_hint: Option<String>,
          cover_art_url: Option<String>,
          duration_ms: Option<u64>,
      },
  }
  ```
* **Reasoning:** Eliminates secondary storage structures for remote streaming tracks. Local files and remote WASM streams share identical queue anatomy, drag-and-drop operations, and shuffle mechanics.
* **Trade-Off:** Remote stream URLs may expire between sessions. On database hydration, remote tracks lacking valid URLs are preserved with metadata for display but pruned if unresolvable on playback.

### 3.2. Independent Shuffle Permutation State
* **Decision:** Shuffle state is encapsulated in an optional struct within `QueueState`:
  ```rust
  pub struct ShuffleState {
      pub order: Vec<String>,       // Shuffled instance_ids
      pub cursor: usize,            // Position in shuffle order
      pub seed: u64,                // PRNG timestamp seed (deterministic seed PRNG tracked in TODO)
      pub regenerate_on_repeat: bool,
  }
  ```
* **Reasoning:** 
  * When shuffle is enabled, all tracks are permuted into a shuffle vector, and `shuffle.cursor` is aligned to the index of the currently active song so playback continues uninterrupted.
  * Moving backward (`prev()`) steps through the deterministic shuffle history rather than picking a random track or jumping to an unexpected array index.
  * When `RepeatMode::All` is active with shuffle, reaching the end of `order` automatically generates a fresh permutation cycle without repeating the last played song back-to-back.
* **Trade-Off:** Requires maintaining synchronization between `tracks` and `shuffle.order` whenever tracks are added, removed, or dragged in the UI (drag-and-drop sync during active shuffle tracked in `TODO.md` under IMPR-03).

### 3.3. Three-Second Previous Playback Rule (Planned Ergonomic Invariant)
* **Design Specification (Tracked in `TODO.md` under FEAT-01):** Pressing "Previous" evaluates current track playback progress:
  * If elapsed time $> 3.0\text{s}$: Restarts current track from $0.0\text{s}$.
  * If elapsed time $\le 3.0\text{s}$: Jumps to the preceding track in the queue (or previous track in shuffle order).
* **Reasoning:** Aligns with standard hardware audio player ergonomics. When listening to a song halfway through, users expect "Previous" to replay the track, not skip backward in the playlist.
* **Current Implementation Note:** In the current build, "Previous" invokes `skip_backward(1)` directly without elapsed-time evaluation.

---

# 4. Playback Invariant Test Matrix (35+ Verified Edge Cases)

The following matrix documents the verified behaviors and mathematical invariants implemented across `src-tauri/src/queue/` and rescued from engineering test notes:

### 4.1. Seek Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **SEEK-01** | Seek past track duration | Clamps position to track duration ($d_{\text{max}}$) and triggers EOF / auto-advance. | Clamped via `Math.min(position, duration)` in frontend player store. |
| **SEEK-02** | Seek to negative timestamp | Clamps position to $0.0\text{s}$. | Clamped via `Math.max(0, position)` in frontend player store. |
| **SEEK-03** | Seek while paused | Position updates across IPC; playback remains in `Paused` state. | State preserved via `was_paused` flag in `AudioCommand::Seek`. |
| **SEEK-04** | Seek on empty player (no track loaded) | Strictly a no-op; returns immediately without error or panic. | Guarded by `if current_track_path.is_empty() { return; }`. |
| **SEEK-05** | Rapid scrubber scrubbing (5 seeks in 200ms) | Last seek wins; stale in-flight seeks are discarded without taking over audio sink. | Guarded by atomic monotonic counter `seek_generation: u64`. |

---

### 4.2. Queue Manipulation Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **QUE-01** | Empty queue + Next / Previous | Returns `Ok(None)`; audio engine remains idle without crashing. | Explicit `if self.tracks.is_empty() { return Ok(None); }`. |
| **QUE-02** | Single-track queue + Next | If `RepeatMode::Off`, stops playback; if `RepeatMode::All` or `One`, restarts track. | Evaluated in `next_normal()` boundary check. |
| **QUE-03** | Unresolvable / Deleted file path | Audio thread catches file open error and logs warning. (Automated advance to next valid item is tracked in `TODO.md` under FEAT-02). | Caught in `SymphoniaSource::from_path` error handling. |
| **QUE-04** | User clicks currently playing track | Restarts track from $0.0\text{s}$ cleanly without duplicating queue entries. | Re-seeks active track or reloads sink. |
| **QUE-05** | Load Album vs. Load Playlist | Both populate `QueueState.tracks` with unique `instance_id` UUIDs and reset cursor. | Handled via `set_queue(tracks, start_index)`. |
| **QUE-06** | Drag-and-drop reorder active track | Active track moves to new index; `current_position` updates to match new index. | Handled in `QueueState::reorder()` cursor update logic. |
| **QUE-07** | Clear queue while playing | If `keep_playing_on_queue_clear = true`, active track finishes; otherwise stops sink. | Evaluated via SQLite user preference in `clear_queue()`. |

---

### 4.3. Skip & Boundary Navigation Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **SKIP-01** | Next at end of queue (`RepeatMode::Off`) | Playback stops cleanly; state transitions to `Stopped`; queue cursor remains at end. | `next_normal()` returns `None`. |
| **SKIP-02** | Next at end of queue (`RepeatMode::All`) | Queue cursor wraps around to index `0`; playback continues seamlessly. | `next_normal()` returns `Some(0)`. |
| **SKIP-03** | Previous at start of queue (index 0) | Replays track from $0.0\text{s}$; zero index underflow. | Handled in `prev_normal()` saturating math. |
| **SKIP-04** | Previous within 3 seconds of playback | Jumps to preceding track in queue sequence. (Planned ergonomic rule tracked in `TODO.md` under FEAT-01). | Currently invokes `skip_backward(1)` directly. |
| **SKIP-05** | Previous after 3 seconds of playback | Re-seeks current track to $0.0\text{s}$. (Planned ergonomic rule tracked in `TODO.md` under FEAT-01). | Currently invokes `skip_backward(1)` directly. |
| **SKIP-06** | Skip Next during active seek operation | In-flight seek thread is superseded by new track load. | `AudioCommand::Seek` increments `seek_generation` (invalidation on `Load` tracked in `TODO.md` under BUG-02). |

---

### 4.4. Shuffle Permutation Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **SHUF-01** | Toggle Shuffle ON mid-playback | Active song remains playing; permutation is generated; shuffle cursor points to active song. | `set_shuffle(true)` aligns `shuffle.cursor` to `current_id` in `shuffle.order`. |
| **SHUF-02** | Toggle Shuffle OFF mid-playback | Original track sequence restored; active song continues without interruption. | `set_shuffle(false)` restores `current_position` to original array index. |
| **SHUF-03** | Shuffle + `RepeatMode::All` cycle end | After all shuffled tracks play, a fresh permutation cycle generates automatically. | `next_shuffle()` triggers `regenerate_shuffle_order()`. |
| **SHUF-04** | Shuffle with single-track queue | Effectively a no-op; returns single item without panic. | Guarded by length checks. |
| **SHUF-05** | Previous while in Shuffle mode | Traverses backward through the deterministic shuffle history vector (`order[cursor - 1]`). | Handled in `prev_shuffle()`. |
| **SHUF-06** | Add track to queue while Shuffle ON | Track appends to `tracks` vector and appends to end of `shuffle.order`. | `add_track()` synchronizes both vectors. |
| **SHUF-07** | Manual reshuffle command | Re-randomizes `shuffle.order` while keeping current track pinned to cursor. | Handled via `commands::reshuffle()`. |

---

### 4.5. Repeat Mode Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **REP-01** | `RepeatMode::One` auto-advance | Track reaches EOF; audio loops on same track. (Mid-queue repeat fix tracked in `TODO.md` under BUG-01). | Handled in `next_normal()` and `next_shuffle()`. |
| **REP-02** | `RepeatMode::One` + Manual Skip Next | Manually pressing "Next" advances forward. | Handled via `skip_forward(1)`. |
| **REP-03** | `RepeatMode::All` + Shuffle active | Queue wraps around to start of newly regenerated shuffle order upon queue exhaustion. | Handled in `next_shuffle()`. |
| **REP-04** | `RepeatMode::Off` + Terminal track EOF | Audio sink empties; `track-ended` event emitted. (UI terminal reset listener tracked in `TODO.md` under BUG-03). | Detected by `sink.empty() && queued_tracks.is_empty()`. |

---

### 4.6. Cross-Feature Interactivity Invariants

| Case ID | Scenario | Expected Invariant & Behavior | Implementation Defense |
| :--- | :--- | :--- | :--- |
| **XFEAT-01** | Shuffle ON + `RepeatMode::One` | `RepeatMode::One` takes precedence; track loops indefinitely until user skips. | `RepeatMode::One` overrides shuffle cursor advance. |
| **XFEAT-02** | Seek to end while `RepeatMode::One` | Seeking to $d_{\text{max}}$ triggers track restart to $0.0\text{s}$, not next track. | Handled by EOF repeat handler. |
| **XFEAT-03** | Load new queue while track playing | Audio sink terminates immediately; old queue flushed; new queue index 0 starts. | `set_queue()` resets queue, followed by `load_audio()` dispatch. |
| **XFEAT-04** | Factory Reset while playing | Audio stops, queue clears, SQLite tables truncate, and UI resets to initial state. | Handled via `DbRequest::FactoryReset` + `AudioCommand::Stop`. |
| **XFEAT-05** | Crash mid-session recovery | On startup, `recover_on_startup()` attempts restoring track list and cursor position. | Automated recovery executed during Tauri initialization (active snapshotting in `TODO.md`). |

---

# 5. Verification & Automated Quality Gates

All queue state operations and navigation edge cases are validated via unit and integration tests:

```bash
# 1. Run Core Queue State Machine Tests (Navigation, Reordering, Shuffle)
cargo test queue::

# 2. Run Queue Module Unit Tests
cargo test queue::tests

# 3. Run Queue SQLite Persistence Tests (Hydration & Serialization)
cargo test queue::persistence::tests

# 4. Stress Test Rapid Track Switching (Scenario S6 Rapid Skip - Requires --full-suite)
./scripts/benchmark-memory.sh --full-suite
# Asserts:
# - thread_cleanup_gate <= 0 (all decoder threads dropped on skip)
# - fd_cleanup_gate <= 3 (no lingering file descriptors)
```
