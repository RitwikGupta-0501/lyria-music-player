# Dual-Mode Infinite Core Loop Architecture

## Overview
Echo's **Dual-Mode Infinite Core Loop** is the automated recommendation and playback engine responsible for continuous, algorithmically-driven audio playback. When the active playback queue nears completion (or reaches the final track), the engine transparently synthesizes the next track and pre-buffers it directly into the backend audio sink for zero-latency gapless transitions.

```
+-------------------------------------------------------------+
|                      Seed Track                             |
|          (Current Playing / Last Queued Track)              |
+------------------------------+------------------------------+
                               |
                               v
             [Autoplay Master Switch Enabled?]
                               |
                +--------------+--------------+
                | Yes                         | No
                v                             v
+-------------------------------+     +-----------------------+
|  Tier 1: Federated WASM Radio |     | Terminate Playback    |
|  - Queries active extensions  |     | (Honor queue setting) |
|  - 3500ms timeout budget      |     +-----------------------+
+---------------+---------------+
                |
     +----------+----------+
     | Success             | Empty / Offline / Timeout
     v                     v
+-----------------+   +------------------------------------+
| Return Remote   |   | Tier 2: Local Markov Random Walk   |
| QueueTrack      |   | - SQLite affinity vector scoring   |
+-----------------+   | - 120-min recency exclusion filter |
                      | - Softmax temperature sampling     |
                      +-----------------+------------------+
                                        |
                             +----------+----------+
                             | Candidates Found    | Library Empty
                             v                     v
                      +-----------------+   +------------------+
                      | Return Local    |   | Terminate Loop   |
                      | QueueTrack      |   | Gracefully       |
                      +-----------------+   +------------------+
```

---

## 1. Mathematical Scoring & Local Markov Walk

The Local Markov Walk computes dynamic transition probabilities across all tracks in the local SQLite database.

### 1.1 Recency Filter
To prevent echo chambers and repetitive loops during long sessions:
$$\text{Filter: } \text{Exclude candidates where } \text{last\_played\_at} \ge \text{datetime('now', '-120 minutes')}$$
*Safety Fallback:* If total candidates after recency exclusion is $< 3$, the filter is relaxed to preserve continuous playback for small libraries.

### 1.2 Affinity Scoring Formula
For each candidate track $t$ against seed $s$:

$$\text{RawScore}(t) = 40 \cdot \text{Sim}_{\text{artist}}(s, t) + 20 \cdot \text{Sim}_{\text{album}}(s, t) + 30 \cdot \text{Sim}_{\text{lexical}}(s, t) + 10 \cdot \text{Sim}_{\text{duration}}(s, t) + \text{Bonus}_{\text{liked}} + \text{Bonus}_{\text{plays}} - \text{Penalty}_{\text{recency}} + \mathcal{N}(0, 5)$$

Where:
- $\text{Sim}_{\text{artist}}(s, t) \in \{0.0, 0.5, 1.0\}$: Exact match ($1.0$) or substring/collaboration ($0.5$).
- $\text{Sim}_{\text{album}}(s, t) \in \{0.0, 1.0\}$: Same album ID or album title match ($1.0$).
- $\text{Sim}_{\text{lexical}}(s, t) \in \{0.0, 0.5\}$: Shared lexical root or title token affinity.
- $\text{Sim}_{\text{duration}}(s, t) = \max\left(0.0, 1.0 - \frac{|\text{dur}_s - \text{dur}_t|}{180\,000\,\text{ms}}\right)$.
- $\text{Bonus}_{\text{liked}} = 15.0$ if user liked the track.
- $\text{Bonus}_{\text{plays}} = \min(15.0, \ln(1 + \text{play\_count}) \times 3.75)$.
- $\text{Penalty}_{\text{recency}} = 20.0 \times \left(1.0 - \frac{\text{hours\_since\_played}}{24.0}\right)$ for tracks played within the last 24 hours.
- $\mathcal{N}(0, 5) \in [0.0, 5.0]$: Random jitter to guarantee non-deterministic exploration.

### 1.3 Top-K Truncation & Softmax Temperature Sampling
1. Rank candidates descending by $\text{RawScore}$ and retain top $K = 15$.
2. Compute Boltzmann distribution with temperature $T = 0.7$:
   $$P(i) = \frac{\exp\left(\frac{\text{Score}(i) - \text{Score}_{\max}}{T \cdot 20.0}\right)}{\sum_{j=1}^{K} \exp\left(\frac{\text{Score}(j) - \text{Score}_{\max}}{T \cdot 20.0}\right)}$$
3. Sample candidate $i \sim P(i)$ using inverse transform sampling (CDF walk).

---

## 2. Federated WASM Extension Waterfall

When third-party WASM extensions implementing `get_radio` or `get_related` capabilities are active:
1. Construct canonical seed representation `CanonicalSeedV1` from current playback metadata.
2. Dispatch concurrent async requests to candidate providers with a strict **3500ms timeout budget**.
3. Deduplicate against current seed track and session history.
4. If a remote recommendation is resolved, transform it into `TrackSourceInfo::Remote` with lazy stream resolution.
5. If remote extensions fail, timeout, or return zero results, **seamlessly fall back to Tier 2 (Local Markov Walk)** without dropping audio frames or interrupting playback.

---

## 3. Gapless Audio Pre-Buffering & Sink Lifecycle

To achieve imperceptible track transitions:
1. When track $N$ is playing, `audioStore.queueNextAudio()` resolves track $N+1$ and calls Tauri IPC `queue_next_audio`.
2. The Rust backend decodes and stages the audio source directly into the `rodio` sink's secondary buffer on a dedicated OS thread.
3. Upon completion of track $N$, `rodio` automatically begins streaming track $N+1$ with **0ms latency**.
4. Rodio emits a `track-advanced` event to Tauri.
5. Frontend advances the queue cursor, triggers `queueNextAudio()`, and resolves track $N+2$.

---

## 4. User Configuration & UI Hygiene

- **Settings Control**: Managed via a master switch in **Settings $\rightarrow$ Audio & Playback $\rightarrow$ Autoplay**.
- **Queue Transparency**: Autoplayed tracks are appended as standard queue items with complete metadata. Per system design directives, no intrusive visual badges or clutter are shown on queue items.
