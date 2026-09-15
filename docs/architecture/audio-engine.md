# Audio Engine & Decoders Architecture

Version: 1.0  
Status: Canonical  
Last Updated: 2026-09-13  
Source Module: [`src-tauri/src/audio/`](../../src-tauri/src/audio/)

---

# 1. Overview: The Subsystem Topology (The What)

The Lyria audio engine is a dedicated, real-time sound generation subsystem engineered for bit-perfect local decoding, low-latency streaming playback, and sample-accurate gapless transitions.

It operates entirely independent of the Tauri async command runtime and frontend DOM event loop, communicating strictly through lock-free message-passing channels.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               TAURI COMMAND LAYER & IPC BRIDGE                                         │
│   commands::load_audio() │ commands::seek_audio() │ commands::set_volume() │ commands::pause_audio()   │
└───────────────────────────────────────────────────┬────────────────────────────────────────────────────┘
                                                    │
                         AudioCommand Channel       │
                         std::sync::mpsc::Receiver  ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               DEDICATED BACKGROUND OS AUDIO THREAD                                     │
│  src-tauri/src/audio/mod.rs: start_audio_thread()                                                      │
│                                                                                                        │
│   ┌────────────────────────────────┐   ┌───────────────────────────────┐   ┌────────────────────────┐  │
│   │       Audio Command Loop       │   │    Track Queue & Metadata     │   │   OS Media Controls    │  │
│   │ - State synchronization        │   │ - QueuedTrackMeta deque       │   │ - Souvlaki (MPRIS/SMTC)│  │
│   │ - Event-driven state IPC sync  │   │ - Gapless advance_rx drain    │   │ - Hardware media keys  │  │
│   │ - Async seek generation gate   │   │ - Duration probing & hints    │   │ - Metadata emission    │  │
│   └───────────────┬────────────────┘   └───────────────┬───────────────┘   └───────────┬────────────┘  │
│                   │                                    │                               │               │
│                   ▼                                    ▼                               │               │
│   ┌────────────────────────────────────────────────────────────────────┐               │               │
│   │                       rodio::Sink & OutputStream                   │               │               │
│   │   - Active TrackedSource<SymphoniaSource>                          │◄──────────────┘               │
│   │   - Preloaded queued TrackedSource<SymphoniaSource> (Gapless)      │                               │
│   └────────────────────────────────┬───────────────────────────────────┘                               │
└────────────────────────────────────┼───────────────────────────────────────────────────────────────────┘
                                     │ Continuous 16-bit Interleaved PCM
                                     ▼
                      ┌─────────────────────────────┐
                      │    ALSA / PulseAudio /      │
                      │  CoreAudio / WASAPI Driver  │
                      └─────────────────────────────┘
```

### Core Responsibilities
1. **Decoder Abstraction:** Uniformly decodes local lossless files (`FLAC`, `WAV`), compressed files (`MP3`, `OGG`, `M4A`), and remote network streams (`Opus`, `HLS`) into continuous 16-bit interleaved PCM.
2. **Deterministic Scheduling:** Guarantees buffer delivery to OS sound cards without dropouts or underruns, regardless of background disk I/O or UI rendering load.
3. **High-Fidelity Resampling:** Real-time arbitrary sample-rate conversion via the `rubato` DSP library using band-limited sinc interpolation with Blackman-Harris windowing.
4. **Gapless Buffer Handoff:** Pre-allocates and feeds the next track's decoded PCM directly into the hardware sink queue, eliminating cross-track latency.
5. **Desktop Media Integration:** Synchronizes playback state and metadata with native OS controls via `souvlaki` (Linux MPRIS D-Bus, Windows SMTC, and macOS NowPlaying).

---

# 2. Motivation & Real-Time Constraints (The Why)

Traditional desktop media players frequently fail in high-workload scenarios due to architectural coupling:

### 2.1. The Dangers of Async Runtime Contention
In naive desktop player implementations, audio decoding and stream ingestion share the same `async` thread pool (e.g. Tokio worker threads) as background database scans, filesystem watchers, JSON serialization, and IPC handling.

When a user triggers an intensive operation—such as importing a 20,000-track library or browsing virtualized grids—Tokio's cooperative work-stealing scheduler experiences task queue latency. If the task responsible for feeding audio buffers is delayed by even **$15\text{ms}$**, the hardware sink buffer starves, causing audible clicks, pops, and micro-stutters.

### 2.2. The Strict Real-Time Audio Contract
To achieve audiophile-grade fidelity, Lyria establishes an invariant audio contract:
* **The Audio Thread Must Never Block on I/O:** The thread interacting with `rodio::Sink` must never perform disk file reads, network HTTP requests, or database queries directly on its execution path.
* **Non-Blocking Underrun Masking:** If a network stream buffer temporarily starves, the decoder yields silence frames (`Some(0)`) rather than blocking the audio pump loop, ensuring the application never deadlocks or hangs.
* **Deterministic Resource Teardown:** Switching or stopping tracks must instantaneously terminate background decoding threads, drop network HTTP connections, and delete transient streaming cache files in `/tmp`.

---

# 3. Key Architectural Decisions & Trade-Offs (The Reasoning)

### 3.1. Dedicated OS Thread vs. Tokio Async Runtime
* **Decision:** Audio sink management and command processing run inside a dedicated OS thread (`std::thread::spawn`), isolated from Tokio.
* **Reasoning:** Operating systems prioritize dedicated OS threads for multimedia scheduling. An isolated thread guarantees sub-millisecond dispatch times for audio commands and PCM buffer feeding, completely decoupling playback from Tokio task load.
* **Trade-Off:** Communication across the Tokio-to-Audio boundary cannot use async/await. It requires explicit, lock-free channel message passing (`std::sync::mpsc::channel`).

### 3.2. Background Decoder Decoupling with In-Memory Ring Buffers
* **Decision:** Each `SymphoniaSource` spawns its own internal decoding thread feeding a lock-free in-memory ring buffer (`ringbuf::HeapRb<i16>`).
* **Reasoning:** Parsing container formats (especially compressed MP4, Opus packets, or Variable Bitrate MP3s) and executing Sinc resampling requires variable CPU time per frame. Decoupling the Symphonia decoding loop into an upstream thread allows it to run ahead of the audio sink, maintaining a steady **4-second PCM reservoir** (`sample_rate * channels * 4`).
* **Trade-Off:** Allocates an in-memory buffer ($\approx 768\text{KB}$ for 48kHz stereo 16-bit PCM) per playing track, which is accounted for in Lyria's strict $\le 25\text{MB}$ host memory budget.

### 3.3. `stream-download` & `StreamMediaSource` for Network Audio
* **Decision:** Remote HTTP audio streams are buffered using the `stream-download` crate (`HttpStream`), wrapped in a custom `StreamMediaSource<T>` implementing Symphonia's `MediaSource` trait.
* **Reasoning:** Symphonia's decoders require a synchronous `Read + Seek` stream with known or discoverable boundaries. Chunked HTTP responses over the network are asynchronous and non-seekable. `stream-download` bridges this gap: it downloads HTTP chunks asynchronously in the background into temporary storage (`TempStorageProvider`), exposing a synchronous, seekable stream with a 32KB prefetch buffer for instantaneous playback start ($<100\text{ms}$).
* **Trade-Off:** Network streams require temporary disk caching in `/tmp`. Lyria enforces automated disk file cleanup upon track completion (verified by benchmark scenario `S7_post_settle`).

### 3.4. Gapless Track Advancement via `TrackedSource`
* **Decision:** Consecutive tracks are pre-appended to the active `rodio::Sink` wrapped in `TrackedSource<S>`, triggering an atomic channel notification (`advance_tx`) when the current stream yields `None` (EOF).
* **Reasoning:** Re-initializing audio hardware between tracks introduces a hardware-level re-negotiation pause ($50\text{ms}-200\text{ms}$). By appending the next track's PCM stream into the active sink before the current track finishes, the hardware audio driver continues reading contiguous samples without interruption.
* **Trade-Off:** Pre-decoding the next track's initial header consumes transient CPU and memory during the final seconds of the preceding track.

### 3.5. Real-Time Resampling via Rubato Sinc Interpolation
* **Decision:** Audio streams with sample rates differing from the hardware target (e.g. 44.1kHz FLAC playing on a 48kHz sound card) are resampled in real-time using `rubato::SincFixedIn<f32>` with Blackman-Harris 2 windowing.
* **Reasoning:** Naive linear interpolation introduces high-frequency aliasing and distortion. Rubato's band-limited sinc interpolation delivers studio-grade conversion with an oversampling factor of 256 and an $f_{\text{cutoff}} = 0.95$.
* **Trade-Off:** High sinc lengths increase CPU usage. Lyria provides configurable sinc lengths (default: 128) and interpolation modes (`Cubic` vs `Linear`) stored in SQLite settings, keeping CPU utilization under $5\%$ (verified by benchmark gate `playback_audio_thread_cpu_pct`).

---

# 4. Implementation Details & Lifecycle (The How)

### 4.1. Module Layout

| File | Purpose & Responsibilities |
| :--- | :--- |
| [`src-tauri/src/audio/mod.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/mod.rs) | Central audio thread loop (`start_audio_thread`), `AudioCommand` dispatch, track queue state machine, seek synchronization, and throttled IPC telemetry. |
| [`src-tauri/src/audio/symphonia_source.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/symphonia_source.rs) | Decoder lifecycle (`SymphoniaSource`), `stream-download` HTTP wrapper, `StreamMediaSource` trait implementation, `ringbuf` thread bridge, and Rubato resampling. |
| [`src-tauri/src/audio/commands.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/commands.rs) | Tauri IPC command bridge exposing playback control (`load_audio`, `seek_audio`, `set_volume`, `pause_audio`, `queue_next_audio`) to Svelte. |
| [`src-tauri/src/audio/hls.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/hls.rs) | HTTP Live Streaming (`.m3u8`) playlist parser, segment resolution, and chunked byte reader. |
| [`src-tauri/src/audio/opus_decoder.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/opus_decoder.rs) | Custom Opus decoder binding native `libopus` into Symphonia's `CodecRegistry`. |
| [`src-tauri/src/audio/media_controls.rs`](file:///home/ritwikg/Repository/echo-desktop/src-tauri/src/audio/media_controls.rs) | Cross-platform OS media integration (`souvlaki`) mapping hardware keys to `AudioCommand` channels. |

---

### 4.2. Audio Command Protocol

The audio thread receives messages through an unbounded MPSC channel:

```rust
pub enum AudioCommand {
    Load {
        source: TrackSource,
        title: String,
        artist: Option<String>,
        album: Option<String>,
        duration_hint: Option<u64>,
    },
    QueueNext {
        source: TrackSource,
        title: String,
        artist: Option<String>,
        album: Option<String>,
        duration_hint: Option<u64>,
    },
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    SetMute(bool),
    SyncState,
    Quit,
}
```

---

### 4.3. Detailed Playback & Decoding Sequence

```mermaid
sequenceDiagram
    autonumber
    participant Svelte as Svelte 5 UI
    participant IPC as Tauri IPC (commands.rs)
    participant AudioThread as Audio Thread (mod.rs)
    participant DecoderThread as Symphonia Decoder Thread
    participant RingBuf as ringbuf::HeapRb<i16>
    participant Sink as rodio::Sink

    Svelte->>IPC: invoke("load_audio", { source, title, ... })
    IPC->>AudioThread: tx.send(AudioCommand::Load { ... })
    
    AudioThread->>AudioThread: Clear queued tracks & reset baselines
    AudioThread->>DecoderThread: SymphoniaSource::open()
    
    activate DecoderThread
    DecoderThread->>RingBuf: Initialize 4-second PCM Ring Buffer
    AudioThread->>Sink: sink.append(TrackedSource::new(source))
    AudioThread->>Sink: sink.play()
    
    loop Real-time Decoding Loop
        DecoderThread->>DecoderThread: format.next_packet()
        DecoderThread->>DecoderThread: decoder.decode(&packet)
        opt Resampling Needed
            DecoderThread->>DecoderThread: rubato.process(samples)
        end
        DecoderThread->>RingBuf: push_slice(interleaved_pcm)
    end

    loop Audio Output Pump
        Sink->>RingBuf: consumer.pop()
        alt Samples Available
            Sink->>Sink: Feed hardware sound card
        else Underrun Occurred
            Sink->>Sink: Yield silence (Some(0)); increment TOTAL_UNDERRUNS
        end
    end

    opt Track Finishes (EOF)
        DecoderThread-->>AudioThread: eof_flag.store(true)
        Sink->>AudioThread: TrackedSource on_end() -> advance_tx.send(())
        AudioThread->>AudioThread: Pop next track from queued_tracks
        AudioThread->>Svelte: emit("track-advanced")
    end
    deactivate DecoderThread
```

---

### 4.4. The Asynchronous Seeking State Machine

Seeking audio streams requires robust concurrency protection to prevent race conditions during rapid scrubber scrubbing:

1. **Monotonic Generation Counter (`seek_generation`):**
   * Every `AudioCommand::Seek(pos)` increments a 64-bit `seek_generation` counter on the audio thread.
   * The current `Sink` is stopped immediately to silence audio output during the seek.
2. **Background Probing & Offset Seek:**
   * A worker thread is spawned to perform container seeking (`SymphoniaSource::from_path_seeked` or `from_url_seeked`).
   * For FLAC files without an embedded `SEEKTABLE`, Symphonia employs `SeekMode::Coarse` bisection search across stream byte offsets.
3. **Stale Seek Discard:**
   * When the seek thread completes, it sends `(generation, position, result, was_paused)` over `seek_tx`.
   * If `generation == seek_generation`, the audio thread installs the seeked source into a fresh `Sink`, resets `sink_pos_baseline`, and restores previous play/pause state.
   * If `generation < seek_generation`, the result is **stale** (the user has scrubbed again) and the decoded source is discarded immediately without taking over the audio sink.

---

### 4.5. High-Frequency Timekeeping & Scrubber Interpolation

To prevent IPC bridge flooding while delivering silky-smooth progress scrubber motion:
* **Decoded Time Calculation:** Position is calculated from elapsed sink playback time adjusted for seek baselines:
  $$\text{Position} = \max(0, \text{raw\_sink\_pos} - \text{sink\_pos\_baseline} + \text{seek\_offset})$$
* **Event-Driven IPC Telemetry (0Hz Steady-State):** Rather than streaming continuous high-frequency ticks across the Tauri IPC bridge, `player-sync` events are dispatched **strictly on discrete state transitions** (`Load`, `Play`, `Pause`, `Stop`, `Seek`, seek completion, and gapless track advance). During uninterrupted playback, IPC overhead is zero.
* **Frontend Clock & Interpolation (4Hz / 250ms Tick):** The Svelte 5 audio store (`audio.svelte.ts`) maintains an internal client-side clock timer via `setInterval` running at 4Hz ($250\text{ms}$). Each tick calculates:
  $$\text{currentTime} = \min\left(\text{syncPosition} + \frac{\text{performance.now()} - \text{syncTimestamp}}{1000}, \text{duration}\right)$$
  This guarantees responsive UI scrubber progression while keeping the Tauri IPC channel completely unencumbered by continuous polling telemetry.

---

# 5. Failure Modes & Edge Case Matrix

| Failure Mode | Root Cause | Engineering Mitigation Strategy |
| :--- | :--- | :--- |
| **Buffer Underrun** | Network latency or slow disk read starves ring buffer. | `SymphoniaSource::next()` catches empty consumer while `eof_flag` is false; logs to `TOTAL_UNDERRUNS` and outputs silence `Some(0)`. **The audio thread never blocks.** |
| **MP4 Demuxer Seek Overflow** | MP4 demuxer seeking backward from EOF panics bounded buffers. | Remote streams use `TempStorageProvider` instead of bounded storage to avoid subtraction overflow panics during backward container seeks. |
| **FLAC Missing Seektable** | Unindexed or raw FLAC file lacking seek metadata. | Engine specifies `SeekMode::Coarse`, triggering Symphonia's internal bisection byte-search algorithm. |
| **Hardware Disconnection** | USB DAC or Bluetooth headphones disconnected during playback. | **Known Gap / Open Issue:** `OutputStream` is initialized once at startup; device drop currently causes silent sink starvation. Planned: Catch device disconnection in audio loop, auto-pause playback, emit notification, and attempt fallback device re-enumeration (tracked in `extensions/TODO.md`). |
| **Orphaned Decoder Threads** | User skips or stops tracks rapidly. | `SymphoniaSource::drop()` sets `abort_flag.store(true, Ordering::Release)`. Background decoding threads inspect this flag every iteration and terminate cleanly within $10\text{ms}$. |

---

# 6. Verification & Automated Testing

Audio engine memory footprints and CPU budgets are continuously verified by empirical benchmark scenarios. Note that benchmarking is **not** equivalent to automated functional tests, and dedicated unit test coverage for the audio subsystem (`cargo test audio::`) is tracked for implementation in `extensions/TODO.md`.

```bash
# 1. Audio Unit Tests (FLAC, MP3, Opus decoding & seeking)
# NOTE: Dedicated unit tests for src-tauri/src/audio are planned (see extensions/TODO.md).
cargo test audio::

# 2. Verify Audio Thread CPU & Memory under Local Playback
./scripts/benchmark-memory.sh --quick

# 3. Assert Threshold Gates (benchmarks/thresholds.json):
# - playback_audio_thread_cpu_pct <= 5.0%
# - playback_anon_growth_mb <= 8.0MB
# - settle_leak_anon_mb <= 5.0MB (no leaked decoder threads)
```
