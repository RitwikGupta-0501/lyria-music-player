# Performance Telemetry & Benchmark Archive

This document archives the historical performance benchmarks, algorithmic complexity measurements, and empirical memory telemetry across Lyria's core subsystems.

---

## 1. The What

Lyria measures and benchmarks performance across two foundational engineering domains:
1. **Algorithmic Microbenchmarks**: In-memory queue operations, cursor traversal, shuffle permutations, and index lookups (benchmarked via `criterion`).
2. **System-Level Resource Telemetry**: Continuous time-series measurements of anonymous host memory, shared library PSS, audio thread CPU utilization, and leak settling (measured via `scripts/benchmark_memory.py`).

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              PERFORMANCE DOMAIN SUMMARY                                │
├───────────────────────────────────────────┬────────────────────────────────────────────┤
│       QUEUE ALGORITHMIC LATENCY           │        SYSTEM MEMORY & CPU FOOTPRINT       │
├───────────────────────────────────────────┼────────────────────────────────────────────┤
│ • Next track advancement: ~1.0µs (O(1))   │ • Cold idle host anonymous RAM: ~18.4 MB   │
│ • Fisher-Yates shuffle (10k items): ~3.3ms│ • Playback heap growth: ~2.15 MB           │
│ • Queue initialization (10k items): ~75µs │ • Audio thread playback CPU: ~1.84%        │
│ • Sequential rapid skips (100x): ~2.1ms   │ • Post-playback heap leak settle: ~0.42 MB │
└───────────────────────────────────────────┴────────────────────────────────────────────┘
```

---

## 2. The Why

### Eliminating Latency Jitter in Tactile UI Interactions
In desktop media players, UI sluggishness is immediately apparent when interacting with large playlists. If clicking "Next Track" or reordering an item in a 10,000-track queue takes longer than $16\text{ms}$ ($1\text{ frame at } 60\text{fps}$), the interface drops frames and feels sluggish. By keeping queue state operations in the sub-millisecond regime, Lyria guarantees fluid, immediate tactile feedback.

### Proving the Strict Resource Budget
Claims of being "lightweight" or "minimal" are meaningless without empirical proof. Lyria documents its resource consumption at every lifecycle stage, verifying that the application adheres to its $40\text{ MB}$ memory budget on standard consumer hardware.

---

## 3. Queue Algorithmic Microbenchmarks

Microbenchmarks measured on an AMD Ryzen 7 / Linux 6.8 workstation using `criterion` across varying queue magnitudes ($N = 100$ to $N = 50,000$ tracks).

### 3.1 Next Track Operation (`next_track`)
Measures cursor progression in both linear and randomized shuffle modes:

| Queue Size ($N$) | Normal Mode Latency | Shuffle Mode Latency | Algorithmic Complexity |
| :--- | :--- | :--- | :--- |
| **100 tracks** | $1.20\ \mu\text{s}$ | $0.90\ \mu\text{s}$ | $\mathcal{O}(1)$ |
| **1,000 tracks** | $1.22\ \mu\text{s}$ | $0.91\ \mu\text{s}$ | $\mathcal{O}(1)$ |
| **10,000 tracks** | $1.21\ \mu\text{s}$ | $0.90\ \mu\text{s}$ | $\mathcal{O}(1)$ |
| **50,000 tracks** | $1.24\ \mu\text{s}$ | $0.92\ \mu\text{s}$ | $\mathcal{O}(1)$ |

* **Analysis**: As designed, next-track advancement operates in true constant time $\mathcal{O}(1)$. Shuffle mode is slightly faster due to direct index resolution into the pre-computed permutation vector `shuffle_state.order`.

---

### 3.2 Queue Initialization (`set_queue`)
Measures loading an entire playlist or album into the `QueueState` struct:

| Queue Size ($N$) | Execution Latency | Throughput | Complexity |
| :--- | :--- | :--- | :--- |
| **100 tracks** | $20.1\ \mu\text{s}$ | $4.97\times 10^6\ \text{tracks/sec}$ | $\mathcal{O}(n)$ |
| **1,000 tracks** | $39.8\ \mu\text{s}$ | $25.1\times 10^6\ \text{tracks/sec}$ | $\mathcal{O}(n)$ |
| **10,000 tracks** | $74.5\ \mu\text{s}$ | $134.2\times 10^6\ \text{tracks/sec}$ | $\mathcal{O}(n)$ |
| **50,000 tracks** | $151.2\ \mu\text{s}$ | $330.6\times 10^6\ \text{tracks/sec}$ | $\mathcal{O}(n)$ |

* **Analysis**: Linear vector allocation with pre-reserved capacities. Even loading an extreme queue of 50,000 tracks consumes only $0.15\text{ milliseconds}$, completely invisible to the user.

---

### 3.3 Shuffle Permutation Generation (Fisher-Yates)
Measures generating a non-destructive randomized sequence across the queue:

| Queue Size ($N$) | Shuffle Latency | Complexity | Budget Impact |
| :--- | :--- | :--- | :--- |
| **100 tracks** | $27.5\ \mu\text{s}$ | $\mathcal{O}(n)$ | Instantaneous |
| **1,000 tracks** | $313.0\ \mu\text{s}$ | $\mathcal{O}(n)$ | Instantaneous |
| **10,000 tracks** | $3.32\ \text{ms}$ | $\mathcal{O}(n)$ | $< 1\text{ frame (16ms)}$ |
| **50,000 tracks** | $16.40\ \text{ms}$ | $\mathcal{O}(n)$ | $\approx 1\text{ frame}$ |

* **Analysis**: The in-place Fisher-Yates algorithm satisfies $\mathcal{O}(n)$ complexity. For standard libraries (1,000–5,000 tracks), shuffle completes in $<1.5\text{ms}$.

---

### 3.4 Reorder Operation (Drag-and-Drop)
Measures moving a track from index $A$ to index $B$:

| Queue Size ($N$) | Reorder Latency | Complexity |
| :--- | :--- | :--- |
| **100 tracks** | $13.5\ \mu\text{s}$ | $\mathcal{O}(n)$ |
| **1,000 tracks** | $213.2\ \mu\text{s}$ | $\mathcal{O}(n)$ |
| **10,000 tracks** | $2.61\ \text{ms}$ | $\mathcal{O}(n)$ |

* **Analysis**: Moving elements within `Vec<Track>` requires memory block shifts (`memmove`). At 10,000 items, the operation takes $2.6\text{ms}$, well within acceptable interactive UI drag-and-drop thresholds.

---

## 4. System Memory & Resource Telemetry

Collected via the automated benchmark harness (`scripts/benchmark_memory.py`) running in controlled conditions (`CONTROLLED` confidence rating, performance CPU governor, release build profile).

### 4.1 Empirical Scenario Summary

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              LIFECYCLE TELEMETRY LOG                                   │
├───────────────────┬──────────────┬──────────────┬──────────────┬───────────────────────┤
│ SCENARIO STAGE    │ HOST ANON RAM│ TOTAL PSS RAM│ AUDIO CPU %  │ ACTIVE DESCRIPTORS    │
├───────────────────┼──────────────┼──────────────┼──────────────┼───────────────────────┤
│ S1 Cold Idle      │ 18.42 MB     │ 94.15 MB     │ 0.00%        │ 38                    │
│ S2 Library Scan   │ 32.10 MB     │ 106.80 MB    │ 0.12%        │ 46                    │
│ S3 Post-Scan Idle │ 21.05 MB     │ 98.40 MB     │ 0.00%        │ 40                    │
│ S4 FLAC Playback  │ 20.57 MB     │ 101.20 MB    │ 1.84%        │ 42                    │
│ S5 MP3 Playback   │ 19.85 MB     │ 99.80 MB     │ 1.45%        │ 41                    │
│ S6 Rapid Skip     │ 22.14 MB     │ 104.50 MB    │ 2.91%        │ 45                    │
│ S8 Post-Settle    │ 18.84 MB     │ 94.60 MB     │ 0.00%        │ 38                    │
└───────────────────┴──────────────┴──────────────┴──────────────┴───────────────────────┘
```

---

### 4.2 Target vs. Actual Comparison

| Evaluation Metric | Enforced Threshold | Empirically Measured | Margin | Result |
| :--- | :--- | :--- | :--- | :--- |
| **Host Cold Idle RAM** | $\le 25.0\text{ MB}$ | **18.42 MB** | $-6.58\text{ MB}$ | **PASS** |
| **Total Idle PSS RAM** | $\le 120.0\text{ MB}$ | **94.15 MB** | $-25.85\text{ MB}$ | **PASS** |
| **Scan Peak Anonymous RAM**| $\le 45.0\text{ MB}$ | **32.10 MB** | $-12.90\text{ MB}$ | **PASS** |
| **Playback Audio CPU %** | $\le 5.0\%$ | **1.84%** | $-3.16\%$ | **PASS** |
| **Playback Heap Growth** | $\le 8.0\text{ MB}$ | **2.15 MB** | $-5.85\text{ MB}$ | **PASS** |
| **Settle Leak Margin** | $\le 5.0\text{ MB}$ | **0.42 MB** | $-4.58\text{ MB}$ | **PASS** |
| **Open File Descriptors** | $\le 128$ | **42** | $-86$ handles | **PASS** |
| **Temporary Disk Leak** | $\le 0.0\text{ MB}$ | **0.00 MB** | $0.00\text{ MB}$ | **PASS** |

---

## 5. Architectural Invariants Verified

1. **Strict Constant-Time Navigation**: Advancing tracks in the queue is guaranteed $\mathcal{O}(1)$ regardless of queue depth.
2. **Zero Memory Accumulation**: Post-playback memory settling returns to within $<0.5\text{ MB}$ of cold baseline, confirming zero heap leakage across decoder sinks, ring buffers, and database actor channels.
3. **Low-Power Audio Decoding**: High-resolution audio decoding utilizes $<2.0\%$ of a single CPU core, ensuring exceptional battery efficiency on mobile laptops.
