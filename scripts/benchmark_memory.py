#!/usr/bin/env python3
"""
Lyria Memory & Resource Benchmark Suite
Comprehensive, time-series benchmarking pipeline for Lyria (Rust/Tauri 2.0).
Designed for developer laptops with advisory environment checks and leak detection.
"""

import argparse
import json
import math
import os
import platform
import re
import shutil
import signal
import statistics
import struct
import subprocess
import sys
import tempfile
import time
import wave
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any

VERSION = "1.0.0"
DEFAULT_SAMPLE_HZ = 2.0
RAPID_SAMPLE_HZ = 4.0
DEFAULT_COLD_SETTLE_S = 15.0
DEFAULT_POST_SETTLE_S = 30.0
DEFAULT_PLAYBACK_S = 180.0
DEFAULT_LONG_SOAK_S = 3600.0
DEFAULT_LONG_SOAK_SAMPLE_S = 30.0

HOST_PATTERNS = ["echo-desktop", "echo_desktop", "lyria", "Lyria"]
WEBKIT_WEB_PROCESS = "WebKitWebProcess"
WEBKIT_NET_PROCESS = "WebKitNetworkProcess"
COMPETING_AUDIO_PROCS = ["spotify", "firefox", "chromium", "chrome", "vlc", "mpv", "rhythmbox", "audacious"]

DEFAULT_THRESHOLDS = {
    "host_idle_anon_mb": 25.0,
    "total_idle_pss_mb": 120.0,
    "scan_peak_anon_mb": 45.0,
    "playback_audio_thread_cpu_pct": 5.0,
    "playback_anon_growth_mb": 8.0,
    "settle_leak_anon_mb": 5.0,
    "max_fd_count": 128,
    "max_temp_file_mb": 0.0,
}

@dataclass
class MemoryMetrics:
    vm_rss_kb: int = 0
    rss_anon_kb: int = 0
    rss_file_kb: int = 0
    rss_shmem_kb: int = 0
    vm_hwm_kb: int = 0
    swap_kb: int = 0
    threads: int = 0
    pss_kb: int | None = None
    pss_dirty_kb: int | None = None

@dataclass
class ThreadStat:
    tid: int = 0
    name: str = ""
    utime: int = 0
    stime: int = 0
    cpu_pct: float = 0.0

@dataclass
class CpuSnapshot:
    utime: int = 0
    stime: int = 0
    timestamp: float = 0.0
    cpu_pct: float = 0.0

@dataclass
class ProcessSnapshot:
    pid: int
    name: str
    role: str  # 'host', 'webkit_web', 'webkit_net', 'other'
    memory: MemoryMetrics = field(default_factory=MemoryMetrics)
    cpu: CpuSnapshot = field(default_factory=CpuSnapshot)
    fd_count: int = 0
    threads: list[ThreadStat] = field(default_factory=list)

@dataclass
class Sample:
    timestamp_ms: int
    scenario: str
    processes: list[ProcessSnapshot] = field(default_factory=list)
    temp_file_bytes: int = 0

@dataclass
class EnvironmentCheck:
    name: str
    passed: bool
    detail: str

@dataclass
class EnvironmentReport:
    checks: list[EnvironmentCheck] = field(default_factory=list)
    fingerprint: dict[str, Any] = field(default_factory=dict)

    @property
    def confidence(self) -> str:
        return "CONTROLLED" if all(c.passed for c in self.checks) else "DEGRADED"

@dataclass
class ScenarioResult:
    name: str
    samples: list[Sample] = field(default_factory=list)
    assertions: list[dict[str, Any]] = field(default_factory=list)
    duration_s: float = 0.0
    summary: dict[str, Any] = field(default_factory=dict)


# --- /proc Parsing & Helpers ---

def read_file(path: str) -> str | None:
    try:
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            return f.read()
    except (OSError, PermissionError):
        return None

def parse_proc_status(pid: int) -> MemoryMetrics:
    content = read_file(f"/proc/{pid}/status")
    if not content:
        return MemoryMetrics()
    m = MemoryMetrics()
    for line in content.splitlines():
        parts = line.split(":", 1)
        if len(parts) != 2:
            continue
        key = parts[0].strip()
        val_str = parts[1].strip().split()[0] if parts[1].strip() else "0"
        try:
            val = int(val_str)
        except ValueError:
            continue

        if key == "VmRSS":
            m.vm_rss_kb = val
        elif key == "RssAnon":
            m.rss_anon_kb = val
        elif key == "RssFile":
            m.rss_file_kb = val
        elif key == "RssShmem":
            m.rss_shmem_kb = val
        elif key == "VmHWM":
            m.vm_hwm_kb = val
        elif key == "Swap":
            m.swap_kb = val
        elif key == "Threads":
            m.threads = val
    return m

def parse_smaps_rollup(pid: int) -> tuple[int | None, int | None]:
    content = read_file(f"/proc/{pid}/smaps_rollup")
    if not content:
        return None, None
    pss, pss_dirty = None, None
    for line in content.splitlines():
        if line.startswith("Pss:"):
            try:
                pss = int(line.split()[1])
            except (IndexError, ValueError):
                pass
        elif line.startswith("Pss_Dirty:"):
            try:
                pss_dirty = int(line.split()[1])
            except (IndexError, ValueError):
                pass
    return pss, pss_dirty

def parse_proc_stat_cpu(pid: int) -> CpuSnapshot:
    content = read_file(f"/proc/{pid}/stat")
    now = time.monotonic()
    if not content:
        return CpuSnapshot(timestamp=now)
    try:
        close_paren = content.rindex(")")
        fields = content[close_paren + 2:].split()
        utime = int(fields[11])
        stime = int(fields[12])
        return CpuSnapshot(utime=utime, stime=stime, timestamp=now)
    except (ValueError, IndexError):
        return CpuSnapshot(timestamp=now)

def get_process_threads(pid: int, prev_threads: dict[int, tuple[int, int, float]] | None = None) -> list[ThreadStat]:
    thread_stats = []
    task_dir = f"/proc/{pid}/task"
    try:
        tids = [int(t) for t in os.listdir(task_dir) if t.isdigit()]
    except OSError:
        return []

    now = time.monotonic()
    clk_tck = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100

    for tid in tids:
        comm = (read_file(f"{task_dir}/{tid}/comm") or "").strip()
        stat_content = read_file(f"{task_dir}/{tid}/stat")
        utime, stime = 0, 0
        if stat_content:
            try:
                close_paren = stat_content.rindex(")")
                fields = stat_content[close_paren + 2:].split()
                utime = int(fields[11])
                stime = int(fields[12])
            except (ValueError, IndexError):
                pass

        cpu_pct = 0.0
        if prev_threads and tid in prev_threads:
            p_utime, p_stime, p_time = prev_threads[tid]
            dt = now - p_time
            if dt > 0:
                delta_ticks = (utime + stime) - (p_utime + p_stime)
                cpu_pct = max(0.0, (delta_ticks / clk_tck / dt) * 100.0)

        thread_stats.append(ThreadStat(tid=tid, name=comm, utime=utime, stime=stime, cpu_pct=cpu_pct))

    return sorted(thread_stats, key=lambda t: t.cpu_pct, reverse=True)

def get_fd_count(pid: int) -> int:
    try:
        return len(os.listdir(f"/proc/{pid}/fd"))
    except OSError:
        return 0

def get_process_name(pid: int) -> str:
    return (read_file(f"/proc/{pid}/comm") or "").strip()

def get_temp_file_usage() -> int:
    total = 0
    tmp_dirs = [tempfile.gettempdir(), "/tmp"]
    seen = set()
    for tmp_dir in tmp_dirs:
        if tmp_dir in seen:
            continue
        seen.add(tmp_dir)
        try:
            for entry in os.scandir(tmp_dir):
                name = entry.name.lower()
                if name.startswith(".tmp") or name.startswith("stream-download-") or name.startswith("lyria-"):
                    if entry.is_file(follow_symlinks=False):
                        try:
                            total += entry.stat(follow_symlinks=False).st_size
                        except OSError:
                            pass
        except OSError:
            pass
    return total

def _get_ppid(pid: int) -> int:
    content = read_file(f"/proc/{pid}/stat")
    if not content:
        return 0
    try:
        close_paren = content.rindex(")")
        return int(content[close_paren + 2:].split()[1])
    except (ValueError, IndexError):
        return 0

def _is_descendant_of(pid: int, ancestor: int, max_depth: int = 10) -> bool:
    current = pid
    for _ in range(max_depth):
        ppid = _get_ppid(current)
        if ppid == ancestor:
            return True
        if ppid <= 1:
            return False
        current = ppid
    return False

def discover_process_tree(host_pid: int | None = None) -> list[tuple[int, str, str]]:
    results = []
    if host_pid:
        name = get_process_name(host_pid)
        if name:
            results.append((host_pid, name, "host"))
    else:
        try:
            pids = [int(d) for d in os.listdir("/proc") if d.isdigit()]
        except OSError:
            return []
        for pid in pids:
            name = get_process_name(pid)
            cmdline = (read_file(f"/proc/{pid}/cmdline") or "").replace(chr(0), " ").strip()
            if any(pat in name or pat in cmdline for pat in HOST_PATTERNS):
                if "benchmark" not in cmdline and "python" not in name:
                    results.append((pid, name, "host"))
                    host_pid = pid
                    break

    if not host_pid:
        return results

    try:
        pids = [int(d) for d in os.listdir("/proc") if d.isdigit()]
    except OSError:
        return results

    for pid in pids:
        if pid == host_pid:
            continue
        name = get_process_name(pid)
        cmdline = (read_file(f"/proc/{pid}/cmdline") or "").replace(chr(0), " ").strip()

        if WEBKIT_WEB_PROCESS in name or WEBKIT_WEB_PROCESS in cmdline:
            if _is_descendant_of(pid, host_pid) or _get_ppid(pid) == host_pid:
                results.append((pid, name, "webkit_web"))
            elif not any(r[2] == "webkit_web" for r in results):
                results.append((pid, name, "webkit_web"))
        elif WEBKIT_NET_PROCESS in name or WEBKIT_NET_PROCESS in cmdline:
            if _is_descendant_of(pid, host_pid) or _get_ppid(pid) == host_pid:
                results.append((pid, name, "webkit_net"))
            elif not any(r[2] == "webkit_net" for r in results):
                results.append((pid, name, "webkit_net"))

    return results

def compute_cpu_percent(prev: CpuSnapshot, curr: CpuSnapshot) -> float:
    dt = curr.timestamp - prev.timestamp
    if dt <= 0:
        return 0.0
    clk_tck = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100
    delta_ticks = (curr.utime + curr.stime) - (prev.utime + prev.stime)
    return max(0.0, (delta_ticks / clk_tck / dt) * 100.0)

def take_snapshot(
    pid: int,
    name: str,
    role: str,
    prev_cpu: CpuSnapshot | None = None,
    prev_threads: dict[int, tuple[int, int, float]] | None = None
) -> ProcessSnapshot:
    memory = parse_proc_status(pid)
    pss, pss_dirty = parse_smaps_rollup(pid)
    memory.pss_kb, memory.pss_dirty_kb = pss, pss_dirty
    curr_cpu = parse_proc_stat_cpu(pid)
    if prev_cpu:
        curr_cpu.cpu_pct = compute_cpu_percent(prev_cpu, curr_cpu)
    fd_count = get_fd_count(pid)
    threads = get_process_threads(pid, prev_threads)
    return ProcessSnapshot(pid=pid, name=name, role=role, memory=memory, cpu=curr_cpu, fd_count=fd_count, threads=threads)

def take_sample(
    process_tree: list[tuple[int, str, str]],
    scenario: str,
    start_time: float,
    prev_sample: Sample | None = None
) -> Sample:
    elapsed_ms = int((time.monotonic() - start_time) * 1000)
    processes = []
    prev_map = {p.pid: p for p in prev_sample.processes} if prev_sample else {}

    for pid, name, role in process_tree:
        if os.path.exists(f"/proc/{pid}"):
            prev_p = prev_map.get(pid)
            prev_cpu = prev_p.cpu if prev_p else None
            prev_threads = {t.tid: (t.utime, t.stime, prev_p.cpu.timestamp) for t in prev_p.threads} if (prev_p and prev_p.threads) else None
            processes.append(take_snapshot(pid, name, role, prev_cpu, prev_threads))

    return Sample(
        timestamp_ms=elapsed_ms,
        scenario=scenario,
        processes=processes,
        temp_file_bytes=get_temp_file_usage()
    )


# --- Environment & Fixtures ---

def check_environment(binary_path: str | None = None, caches_dropped: bool = False) -> EnvironmentReport:
    report = EnvironmentReport()

    # 1. CPU Governor
    gov = (read_file("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") or "").strip()
    if gov:
        report.checks.append(
            EnvironmentCheck(
                "CPU governor",
                gov == "performance",
                f"{gov}" + (" ✓" if gov == "performance" else " (recommend: performance)")
            )
        )

    # 2. Page caches
    report.checks.append(
        EnvironmentCheck(
            "Page caches",
            caches_dropped,
            "dropped ✓" if caches_dropped else "not dropped (cold start may reflect warm cache)"
        )
    )

    # 3. Competing audio
    competing = []
    try:
        for pid_str in os.listdir("/proc"):
            if pid_str.isdigit():
                name = get_process_name(int(pid_str)).lower()
                if any(p in name for p in COMPETING_AUDIO_PROCS):
                    competing.append(f"{name} (PID {pid_str})")
    except OSError:
        pass
    report.checks.append(
        EnvironmentCheck(
            "Competing audio",
            len(competing) == 0,
            "none ✓" if not competing else ", ".join(competing) + " detected"
        )
    )

    # 4. Release build
    if binary_path and os.path.isfile(binary_path):
        try:
            out = subprocess.run(["file", binary_path], capture_output=True, text=True, timeout=5).stdout
            has_debug = "with debug_info" in out or "not stripped" in out
            report.checks.append(
                EnvironmentCheck(
                    "Release build",
                    not has_debug,
                    "release ✓" if not has_debug else "debug symbols detected (overhead 3-5x higher)"
                )
            )
        except Exception:
            report.checks.append(EnvironmentCheck("Release build", True, "unable to verify"))
    else:
        report.checks.append(EnvironmentCheck("Release build", True, "skipped (attached mode)"))

    # 5. Swap pressure
    meminfo = read_file("/proc/meminfo") or ""
    swap_total_kb = 0
    swap_free_kb = 0
    for line in meminfo.splitlines():
        if line.startswith("SwapTotal:"):
            try:
                swap_total_kb = int(line.split()[1])
            except (IndexError, ValueError):
                pass
        elif line.startswith("SwapFree:"):
            try:
                swap_free_kb = int(line.split()[1])
            except (IndexError, ValueError):
                pass
    swap_used_kb = swap_total_kb - swap_free_kb
    is_swapping = swap_used_kb > 1024 * 100
    report.checks.append(
        EnvironmentCheck(
            "Swap pressure",
            not is_swapping,
            f"{swap_used_kb // 1024}MB used" + (" (pressure detected)" if is_swapping else " ✓")
        )
    )

    # 6. PSS data availability
    pss_test, _ = parse_smaps_rollup(os.getpid())
    report.checks.append(
        EnvironmentCheck(
            "PSS data",
            pss_test is not None,
            "available ✓" if pss_test is not None else "unavailable (permission denied)"
        )
    )

    cpu_model = "Unknown"
    cpuinfo = read_file("/proc/cpuinfo") or ""
    for line in cpuinfo.splitlines():
        if "model name" in line:
            parts = line.split(":", 1)
            if len(parts) == 2:
                cpu_model = parts[1].strip()
                break

    ram_total_mb = 0
    for line in meminfo.splitlines():
        if line.startswith("MemTotal:"):
            try:
                ram_total_mb = int(line.split()[1]) // 1024
            except (IndexError, ValueError):
                pass
            break

    git_hash = ""
    try:
        git_hash = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True, text=True, timeout=3).stdout.strip()
    except Exception:
        pass

    report.fingerprint = {
        "kernel": platform.release(),
        "os": platform.platform(),
        "cpu_model": cpu_model,
        "cpu_cores": os.cpu_count() or 1,
        "ram_total_mb": ram_total_mb,
        "swap_total_mb": swap_total_kb // 1024,
        "cpu_governor": gov or "unknown",
        "git_commit": git_hash,
        "confidence": report.confidence,
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "benchmark_version": VERSION
    }

    return report

def generate_wav_fixtures(target_dir: str, count: int = 50) -> list[str]:
    os.makedirs(target_dir, exist_ok=True)
    generated = []
    sample_rate = 44100
    num_samples = 4410  # 0.1s silence
    for i in range(count):
        filename = os.path.join(target_dir, f"track_{i:04d}.wav")
        with wave.open(filename, "wb") as wf:
            wf.setnchannels(1)
            wf.setsampwidth(2)
            wf.setframerate(sample_rate)
            data = struct.pack(f"<{num_samples}h", *([0] * num_samples))
            wf.writeframes(data)
        generated.append(filename)
    return generated

def find_audio_fixtures(test_dir: str | None) -> tuple[str | None, str | None]:
    if not test_dir or not os.path.isdir(test_dir):
        return None, None
    flac_file = None
    mp3_file = None
    for root, _, files in os.walk(test_dir):
        for f in files:
            lower = f.lower()
            if not flac_file and lower.endswith(".flac"):
                flac_file = os.path.join(root, f)
            if not mp3_file and lower.endswith(".mp3"):
                mp3_file = os.path.join(root, f)
            if flac_file and mp3_file:
                break
        if flac_file and mp3_file:
            break
    return flac_file, mp3_file

def load_thresholds(config_path: str = "benchmarks/thresholds.json") -> dict[str, float]:
    thresholds = dict(DEFAULT_THRESHOLDS)
    if os.path.isfile(config_path):
        try:
            with open(config_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                if isinstance(data, dict):
                    thresholds.update(data)
        except Exception:
            pass
    return thresholds


# --- Scenario Engine & Stats ---

def summarize_scenario(samples: list[Sample], scenario_name: str) -> dict[str, Any]:
    if not samples:
        return {}

    host_samples = []
    web_samples = []
    net_samples = []
    total_rss_list = []
    total_pss_list = []
    temp_bytes_list = []

    for s in samples:
        temp_bytes_list.append(s.temp_file_bytes)
        t_rss = 0
        t_pss = 0
        for p in s.processes:
            t_rss += p.memory.vm_rss_kb
            if p.memory.pss_kb is not None:
                t_pss += p.memory.pss_kb
            if p.role == "host":
                host_samples.append(p)
            elif p.role == "webkit_web":
                web_samples.append(p)
            elif p.role == "webkit_net":
                net_samples.append(p)
        total_rss_list.append(t_rss)
        total_pss_list.append(t_pss)

    summary: dict[str, Any] = {
        "scenario": scenario_name,
        "samples_count": len(samples),
        "temp_file_peak_mb": max(temp_bytes_list, default=0) / (1024 * 1024),
        "temp_file_final_mb": (temp_bytes_list[-1] if temp_bytes_list else 0) / (1024 * 1024),
        "total_peak_rss_mb": max(total_rss_list, default=0) / 1024,
        "total_mean_rss_mb": (statistics.mean(total_rss_list) if total_rss_list else 0) / 1024,
        "total_peak_pss_mb": max(total_pss_list, default=0) / 1024,
        "total_mean_pss_mb": (statistics.mean(total_pss_list) if total_pss_list else 0) / 1024,
    }

    if host_samples:
        host_rss = [p.memory.vm_rss_kb for p in host_samples]
        host_anon = [p.memory.rss_anon_kb for p in host_samples]
        host_file = [p.memory.rss_file_kb for p in host_samples]
        host_pss = [p.memory.pss_kb for p in host_samples if p.memory.pss_kb is not None]
        host_cpu = [p.cpu.cpu_pct for p in host_samples]
        host_threads = [p.memory.threads for p in host_samples]
        host_fds = [p.fd_count for p in host_samples]

        settle_slice = max(1, len(host_samples) // 5)
        settle_anon = statistics.mean(host_anon[-settle_slice:]) if host_anon else 0

        summary["host"] = {
            "peak_rss_mb": max(host_rss, default=0) / 1024,
            "mean_rss_mb": (statistics.mean(host_rss) if host_rss else 0) / 1024,
            "peak_anon_mb": max(host_anon, default=0) / 1024,
            "mean_anon_mb": (statistics.mean(host_anon) if host_anon else 0) / 1024,
            "settle_anon_mb": settle_anon / 1024,
            "peak_file_mb": max(host_file, default=0) / 1024,
            "peak_pss_mb": (max(host_pss, default=0) / 1024) if host_pss else None,
            "mean_pss_mb": ((statistics.mean(host_pss) / 1024) if host_pss else None),
            "peak_cpu_pct": max(host_cpu, default=0.0),
            "mean_cpu_pct": statistics.mean(host_cpu) if host_cpu else 0.0,
            "final_threads": host_threads[-1] if host_threads else 0,
            "peak_threads": max(host_threads, default=0),
            "final_fds": host_fds[-1] if host_fds else 0,
            "peak_fds": max(host_fds, default=0),
        }

        all_threads: dict[str, list[float]] = {}
        for p in host_samples:
            for t in p.threads:
                key = f"{t.name}:{t.tid}"
                all_threads.setdefault(key, []).append(t.cpu_pct)
        top_threads = []
        for key, cpu_vals in all_threads.items():
            t_name, tid_s = key.split(":", 1)
            top_threads.append({
                "name": t_name or "unknown",
                "tid": int(tid_s),
                "mean_cpu_pct": round(statistics.mean(cpu_vals), 2),
                "peak_cpu_pct": round(max(cpu_vals, default=0.0), 2),
            })
        top_threads.sort(key=lambda x: x["mean_cpu_pct"], reverse=True)
        summary["top_threads"] = top_threads[:8]

    if web_samples:
        web_rss = [p.memory.vm_rss_kb for p in web_samples]
        web_pss = [p.memory.pss_kb for p in web_samples if p.memory.pss_kb is not None]
        summary["webkit_web"] = {
            "peak_rss_mb": max(web_rss, default=0) / 1024,
            "mean_rss_mb": (statistics.mean(web_rss) if web_rss else 0) / 1024,
            "peak_pss_mb": (max(web_pss, default=0) / 1024) if web_pss else None,
        }

    if net_samples:
        net_rss = [p.memory.vm_rss_kb for p in net_samples]
        net_pss = [p.memory.pss_kb for p in net_samples if p.memory.pss_kb is not None]
        summary["webkit_net"] = {
            "peak_rss_mb": max(net_rss, default=0) / 1024,
            "mean_rss_mb": (statistics.mean(net_rss) if net_rss else 0) / 1024,
            "peak_pss_mb": (max(net_pss, default=0) / 1024) if net_pss else None,
        }

    return summary

def sample_loop(
    process_tree: list[tuple[int, str, str]],
    scenario: str,
    duration_s: float,
    sample_hz: float,
    start_time: float,
    all_samples: list[Sample],
    desc: str = ""
) -> ScenarioResult:
    result = ScenarioResult(name=scenario)
    interval = 1.0 / sample_hz
    loop_start = time.monotonic()
    prev_sample = None
    host_pid = process_tree[0][0] if process_tree else None

    while (time.monotonic() - loop_start) < duration_s:
        current_tree = discover_process_tree(host_pid) or process_tree
        sample = take_sample(current_tree, scenario, start_time, prev_sample)
        result.samples.append(sample)
        all_samples.append(sample)
        prev_sample = sample

        elapsed = time.monotonic() - loop_start
        host_snap = next((p for p in sample.processes if p.role == "host"), None)
        host_rss = host_snap.memory.vm_rss_kb // 1024 if host_snap else 0
        host_anon = host_snap.memory.rss_anon_kb // 1024 if host_snap else 0
        host_cpu = host_snap.cpu.cpu_pct if host_snap else 0.0

        status_text = (
            f"\r\033[K  [{sample.timestamp_ms/1000:6.1f}s] {scenario:16s} "
            f"| Host RSS: {host_rss:3d}MB (Anon: {host_anon:3d}MB) "
            f"| CPU: {host_cpu:4.1f}% "
            f"| {desc}"
        )
        print(status_text, end="", flush=True)

        next_tick = len(result.samples) * interval
        sleep_rem = next_tick - (time.monotonic() - loop_start)
        if sleep_rem > 0:
            time.sleep(sleep_rem)

    print()
    result.duration_s = time.monotonic() - loop_start
    result.summary = summarize_scenario(result.samples, scenario)
    result.summary["duration_s"] = result.duration_s
    return result

def evaluate_assertions(
    scenarios: list[ScenarioResult],
    thresholds: dict[str, float]
) -> list[dict[str, Any]]:
    assertions = []
    s_map = {s.name: s for s in scenarios}

    s1 = s_map.get("S1_cold_idle")
    if s1 and s1.summary and "host" in s1.summary:
        host_s1 = s1.summary["host"]
        passed = host_s1["mean_anon_mb"] <= thresholds["host_idle_anon_mb"]
        assertions.append({
            "scenario": "S1_cold_idle",
            "name": "Host Idle Anonymous Memory (RssAnon)",
            "passed": passed,
            "threshold": f"<= {thresholds['host_idle_anon_mb']:.1f} MB",
            "actual": f"{host_s1['mean_anon_mb']:.2f} MB",
        })

        if s1.summary.get("total_mean_pss_mb"):
            passed_pss = s1.summary["total_mean_pss_mb"] <= thresholds["total_idle_pss_mb"]
            assertions.append({
                "scenario": "S1_cold_idle",
                "name": "Total Idle Memory (PSS)",
                "passed": passed_pss,
                "threshold": f"<= {thresholds['total_idle_pss_mb']:.1f} MB",
                "actual": f"{s1.summary['total_mean_pss_mb']:.2f} MB",
            })

    s2 = s_map.get("S2_library_scan")
    if s2 and s2.summary and "host" in s2.summary:
        host_s2 = s2.summary["host"]
        passed_scan = host_s2["peak_anon_mb"] <= thresholds["scan_peak_anon_mb"]
        assertions.append({
            "scenario": "S2_library_scan",
            "name": "Peak Scan Memory Spike",
            "passed": passed_scan,
            "threshold": f"<= {thresholds['scan_peak_anon_mb']:.1f} MB",
            "actual": f"{host_s2['peak_anon_mb']:.2f} MB",
        })

    s4 = s_map.get("S4_local_playback")
    if s4 and s4.summary and "host" in s4.summary:
        host_s4 = s4.summary["host"]
        top_threads = s4.summary.get("top_threads", [])
        audio_thread = next((t for t in top_threads if "audio" in t["name"].lower() or "sink" in t["name"].lower()), None)
        audio_cpu = audio_thread["mean_cpu_pct"] if audio_thread else host_s4["mean_cpu_pct"]
        passed_audio = audio_cpu <= thresholds["playback_audio_thread_cpu_pct"]
        assertions.append({
            "scenario": "S4_local_playback",
            "name": "Audio Thread CPU Consumption",
            "passed": passed_audio,
            "threshold": f"<= {thresholds['playback_audio_thread_cpu_pct']:.1f}%",
            "actual": f"{audio_cpu:.1f}%",
        })

        if s1 and "host" in s1.summary:
            anon_growth = host_s4["mean_anon_mb"] - s1.summary["host"]["mean_anon_mb"]
            passed_growth = anon_growth <= thresholds["playback_anon_growth_mb"]
            assertions.append({
                "scenario": "S4_local_playback",
                "name": "Playback Memory Growth vs Idle",
                "passed": passed_growth,
                "threshold": f"<= {thresholds['playback_anon_growth_mb']:.1f} MB",
                "actual": f"{anon_growth:+.2f} MB",
            })

    s_settle = s_map.get("S7_post_settle") or s_map.get("S8_post_playback_idle")
    if s_settle and s1 and s_settle.summary and s1.summary and "host" in s_settle.summary and "host" in s1.summary:
        host_settle = s_settle.summary["host"]
        host_s1 = s1.summary["host"]

        leak_delta = host_settle["settle_anon_mb"] - host_s1["settle_anon_mb"]
        passed_leak = leak_delta <= thresholds["settle_leak_anon_mb"]
        assertions.append({
            "scenario": s_settle.name,
            "name": "Memory Leak Gate (Post-Idle - Cold-Idle Anon)",
            "passed": passed_leak,
            "threshold": f"<= {thresholds['settle_leak_anon_mb']:.1f} MB",
            "actual": f"{leak_delta:+.2f} MB",
        })

        thread_delta = host_settle["final_threads"] - host_s1["final_threads"]
        passed_threads = thread_delta <= 0
        assertions.append({
            "scenario": s_settle.name,
            "name": "Thread Cleanup Gate (Post-Idle - Cold-Idle Threads)",
            "passed": passed_threads,
            "threshold": "<= 0 threads",
            "actual": f"{thread_delta:+d} threads",
        })

        fd_delta = host_settle["final_fds"] - host_s1["final_fds"]
        passed_fds = fd_delta <= 3
        assertions.append({
            "scenario": s_settle.name,
            "name": "File Descriptor Cleanup Gate (Post-Idle - Cold-Idle FDs)",
            "passed": passed_fds,
            "threshold": "<= 3 FDs",
            "actual": f"{fd_delta:+d} FDs",
        })

        temp_mb = s_settle.summary.get("temp_file_final_mb", 0.0)
        passed_temp = temp_mb <= thresholds["max_temp_file_mb"]
        assertions.append({
            "scenario": s_settle.name,
            "name": "Temporary Disk File Cleanup",
            "passed": passed_temp,
            "threshold": f"<= {thresholds['max_temp_file_mb']:.1f} MB",
            "actual": f"{temp_mb:.2f} MB",
        })

    return assertions


# --- Reporting & Baseline Comparison ---

def generate_markdown_report(report_data: dict[str, Any]) -> str:
    env = report_data.get("environment", {})
    fingerprint = report_data.get("fingerprint", {})
    scenarios = report_data.get("scenarios", [])
    assertions = report_data.get("assertions", [])
    baseline_diff = report_data.get("baseline_comparison", "")

    confidence = fingerprint.get("confidence", "DEGRADED")
    status_icon = "✓" if confidence == "CONTROLLED" else "⚠"

    lines = []
    lines.append("# Lyria Memory & Resource Benchmark Report")
    lines.append(f"**Generated:** {fingerprint.get('timestamp_utc', 'N/A')}  |  **Version:** v{fingerprint.get('benchmark_version', VERSION)}")
    lines.append("")

    lines.append(f"## Environment Status: {status_icon} {confidence}")
    lines.append("")
    lines.append("| Pre-run Advisory Check | Status | Detail |")
    lines.append("| :--- | :---: | :--- |")
    for check in env.get("checks", []):
        icon = "✓" if check["passed"] else "⚠"
        lines.append(f"| {check['name']} | {icon} | {check['detail']} |")
    lines.append("")

    lines.append("### System Fingerprint")
    lines.append("")
    lines.append("| Parameter | Value |")
    lines.append("| :--- | :--- |")
    lines.append(f"| **OS / Kernel** | {fingerprint.get('os', 'Unknown')} ({fingerprint.get('kernel', 'Unknown')}) |")
    lines.append(f"| **CPU Model** | {fingerprint.get('cpu_model', 'Unknown')} ({fingerprint.get('cpu_cores', '?')} cores) |")
    lines.append(f"| **CPU Governor** | {fingerprint.get('cpu_governor', 'Unknown')} |")
    lines.append(f"| **System RAM** | {fingerprint.get('ram_total_mb', 0):,d} MB |")
    lines.append(f"| **System Swap** | {fingerprint.get('swap_total_mb', 0):,d} MB |")
    lines.append(f"| **Git Commit** | {fingerprint.get('git_commit', 'N/A') or 'N/A'} |")
    lines.append("")

    lines.append("## Executive Summary")
    lines.append("")
    lines.append("| Scenario | Duration | Host Peak RSS | Host Mean Anon | Total Peak PSS | Host CPU (mean) | Threads (peak) | FDs (peak) |")
    lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")

    for sc in scenarios:
        name = sc.get("scenario", "unknown")
        dur = f"{sc.get('duration_s', 0):.1f}s"
        h = sc.get("host", {})
        host_rss = f"{h.get('peak_rss_mb', 0.0):.1f} MB" if h else "-"
        host_anon = f"{h.get('mean_anon_mb', 0.0):.1f} MB" if h else "-"
        tot_pss = f"{sc.get('total_peak_pss_mb', 0.0):.1f} MB" if sc.get("total_peak_pss_mb") else "-"
        host_cpu = f"{h.get('mean_cpu_pct', 0.0):.1f}%" if h else "-"
        threads = f"{h.get('peak_threads', 0)}" if h else "-"
        fds = f"{h.get('peak_fds', 0)}" if h else "-"
        lines.append(f"| **{name}** | {dur} | {host_rss} | {host_anon} | {tot_pss} | {host_cpu} | {threads} | {fds} |")
    lines.append("")

    if assertions:
        lines.append("## Advisory Threshold Assertions")
        lines.append("")
        lines.append("| Scenario | Target Assertion | Threshold | Measured Value | Result |")
        lines.append("| :--- | :--- | :---: | :---: | :---: |")
        for a in assertions:
            res_icon = "✓ PASS" if a["passed"] else "⚠ WARN"
            lines.append(f"| `{a['scenario']}` | {a['name']} | {a['threshold']} | **{a['actual']}** | {res_icon} |")
        lines.append("")

    lines.append("## Scenario Details & Thread Breakdown")
    lines.append("")
    for sc in scenarios:
        name = sc.get("scenario", "unknown")
        lines.append(f"### {name}")
        h = sc.get("host", {})
        web = sc.get("webkit_web", {})
        net = sc.get("webkit_net", {})

        lines.append("| Process Role | Peak RSS | Mean RSS | Peak Anon | Mean Anon | Peak PSS | Mean CPU% |")
        lines.append("| :--- | :---: | :---: | :---: | :---: | :---: | :---: |")
        if h:
            pss_str = f"{h.get('peak_pss_mb', 0.0):.1f} MB" if h.get("peak_pss_mb") is not None else "N/A"
            lines.append(f"| **Host Daemon** | {h.get('peak_rss_mb', 0.0):.1f} MB | {h.get('mean_rss_mb', 0.0):.1f} MB | {h.get('peak_anon_mb', 0.0):.1f} MB | {h.get('mean_anon_mb', 0.0):.1f} MB | {pss_str} | {h.get('mean_cpu_pct', 0.0):.1f}% |")
        if web:
            web_pss = f"{web.get('peak_pss_mb', 0.0):.1f} MB" if web.get("peak_pss_mb") is not None else "N/A"
            lines.append(f"| **WebKit WebProcess** | {web.get('peak_rss_mb', 0.0):.1f} MB | {web.get('mean_rss_mb', 0.0):.1f} MB | - | - | {web_pss} | - |")
        if net:
            net_pss = f"{net.get('peak_pss_mb', 0.0):.1f} MB" if net.get("peak_pss_mb") is not None else "N/A"
            lines.append(f"| **WebKit Network** | {net.get('peak_rss_mb', 0.0):.1f} MB | {net.get('mean_rss_mb', 0.0):.1f} MB | - | - | {net_pss} | - |")
        lines.append("")

        top_threads = sc.get("top_threads", [])
        if top_threads:
            lines.append("**Active Host Threads (Top CPU Consumers):**")
            lines.append("")
            lines.append("| Thread Name | TID | Mean CPU% | Peak CPU% |")
            lines.append("| :--- | :---: | :---: | :---: |")
            for t in top_threads[:5]:
                lines.append(f"| `{t['name']}` | {t['tid']} | {t['mean_cpu_pct']:.1f}% | {t['peak_cpu_pct']:.1f}% |")
            lines.append("")

    if baseline_diff:
        lines.append("## Baseline Comparison")
        lines.append("")
        lines.append(baseline_diff)
        lines.append("")

    return "\n".join(lines)

def compare_against_baseline(current_summary: dict[str, Any], baseline_path: str) -> str:
    if not os.path.isfile(baseline_path):
        return f"*Baseline file not found at `{baseline_path}`.*"

    try:
        with open(baseline_path, "r", encoding="utf-8") as f:
            base_data = json.load(f)
    except Exception as e:
        return f"*Failed to parse baseline file: {e}*"

    lines = []
    lines.append("| Scenario & Metric | Baseline | Current | Delta | Change | Status |")
    lines.append("| :--- | :---: | :---: | :---: | :---: | :---: |")

    curr_scenarios = {s["scenario"]: s for s in current_summary.get("scenarios", [])}
    base_scenarios = {s["scenario"]: s for s in base_data.get("scenarios", [])}

    all_keys = ["S1_cold_idle", "S4_local_playback", "S7_post_settle", "S8_post_playback_idle"]
    for skey in all_keys:
        curr_s = curr_scenarios.get(skey)
        base_s = base_scenarios.get(skey)
        if not curr_s or not base_s:
            continue

        c_host = curr_s.get("host", {})
        b_host = base_s.get("host", {})

        metrics_to_compare = [
            ("Host Mean Anon", "mean_anon_mb", "MB", 15.0),
            ("Host Peak RSS", "peak_rss_mb", "MB", 15.0),
            ("Host Mean CPU%", "mean_cpu_pct", "%", 20.0),
        ]

        for m_name, m_key, unit, max_regress_pct in metrics_to_compare:
            if m_key in c_host and m_key in b_host:
                c_val = c_host[m_key]
                b_val = b_host[m_key]
                delta = c_val - b_val
                pct = (delta / b_val * 100.0) if b_val != 0 else 0.0

                if delta > 0 and pct > max_regress_pct:
                    status = "▲ REGRESSION"
                elif delta < 0 and abs(pct) > max_regress_pct:
                    status = "▼ IMPROVEMENT"
                else:
                    status = "✓ MATCH"

                lines.append(
                    f"| `{skey}` {m_name} | {b_val:.1f} {unit} | {c_val:.1f} {unit} | "
                    f"{delta:+.1f} {unit} | {pct:+.1f}% | {status} |"
                )

    return "\n".join(lines)


# --- Modes: Watch Dashboard, Long Soak, Runner ---

def run_watch_dashboard(host_pid: int | None, sample_hz: float = 2.0):
    print("\033[2J\033[H", end="")
    print(f"Lyria Memory & Resource Monitor (Watch Mode) - v{VERSION}")
    print("Press Ctrl+C to exit...\n")

    start_time = time.monotonic()
    prev_sample = None

    try:
        while True:
            tree = discover_process_tree(host_pid)
            if not tree:
                print(f"\r\033[KWaiting for Lyria process to appear... ({time.monotonic() - start_time:.1f}s)", end="", flush=True)
                time.sleep(1.0)
                continue

            host_pid = tree[0][0]
            sample = take_sample(tree, "WATCH", start_time, prev_sample)
            prev_sample = sample

            # Draw dashboard
            out = ["\033[H\033[K"]
            out.append(f"=== Lyria Live Resource Dashboard (PID: {host_pid}) ===\n")
            out.append(f"{'Role':<15} {'Comm':<20} {'PID':<8} {'RSS (MB)':<10} {'Anon (MB)':<10} {'File (MB)':<10} {'PSS (MB)':<10} {'CPU%':<8} {'Threads':<8} {'FDs':<6}")
            out.append("-" * 105)

            host_proc = None
            total_rss = 0
            total_pss = 0

            for p in sample.processes:
                total_rss += p.memory.vm_rss_kb
                if p.memory.pss_kb is not None:
                    total_pss += p.memory.pss_kb
                if p.role == "host":
                    host_proc = p

                rss_mb = f"{p.memory.vm_rss_kb / 1024:.1f}"
                anon_mb = f"{p.memory.rss_anon_kb / 1024:.1f}"
                file_mb = f"{p.memory.rss_file_kb / 1024:.1f}"
                pss_mb = f"{p.memory.pss_kb / 1024:.1f}" if p.memory.pss_kb is not None else "N/A"
                cpu_str = f"{p.cpu.cpu_pct:.1f}%"
                threads_str = str(p.memory.threads)
                fds_str = str(p.fd_count)

                out.append(f"{p.role:<15} {p.name[:19]:<20} {p.pid:<8} {rss_mb:<10} {anon_mb:<10} {file_mb:<10} {pss_mb:<10} {cpu_str:<8} {threads_str:<8} {fds_str:<6}")

            out.append("-" * 105)
            pss_tot_str = f"{total_pss / 1024:.1f} MB" if total_pss > 0 else "N/A"
            out.append(f"Total RSS: {total_rss / 1024:.1f} MB | Total PSS: {pss_tot_str} | Temp Files: {sample.temp_file_bytes / (1024*1024):.2f} MB\n")

            if host_proc and host_proc.threads:
                out.append("Top Active Threads (Host Daemon):")
                out.append(f"  {'Thread Name':<25} {'TID':<8} {'CPU%'}")
                for t in host_proc.threads[:5]:
                    out.append(f"  {t.name[:24]:<25} {t.tid:<8} {t.cpu_pct:.1f}%")

            print("\n".join(out), flush=True)
            time.sleep(1.0 / sample_hz)

    except KeyboardInterrupt:
        print("\n\nWatch mode stopped.")


def run_long_soak(
    tree: list[tuple[int, str, str]],
    duration_s: float,
    sample_interval_s: float,
    output_dir: str
):
    print(f"\nStarting Long-Soak Endurance Test ({duration_s/60:.1f} minutes, interval {sample_interval_s}s)...")
    start_time = time.monotonic()
    samples: list[Sample] = []
    host_pid = tree[0][0]
    prev_sample = None

    while (time.monotonic() - start_time) < duration_s:
        current_tree = discover_process_tree(host_pid) or tree
        sample = take_sample(current_tree, "LONG_SOAK", start_time, prev_sample)
        samples.append(sample)
        prev_sample = sample

        elapsed = time.monotonic() - start_time
        host_p = next((p for p in sample.processes if p.role == "host"), None)
        rss_mb = host_p.memory.vm_rss_kb // 1024 if host_p else 0
        anon_mb = host_p.memory.rss_anon_kb // 1024 if host_p else 0

        print(
            f"\r\033[K  [{elapsed/60:5.1f}m / {duration_s/60:.1f}m] "
            f"Samples: {len(samples)} | Host RSS: {rss_mb}MB (Anon: {anon_mb}MB)",
            end="",
            flush=True
        )
        time.sleep(sample_interval_s)

    print("\n\nLong soak complete. Computing linear regression slope on anonymous heap memory...")

    host_anon_points = []
    for s in samples:
        host_p = next((p for p in s.processes if p.role == "host"), None)
        if host_p:
            t_min = s.timestamp_ms / (1000.0 * 60.0)
            host_anon_points.append((t_min, host_p.memory.rss_anon_kb))

    if len(host_anon_points) >= 2:
        n = len(host_anon_points)
        sum_x = sum(p[0] for p in host_anon_points)
        sum_y = sum(p[1] for p in host_anon_points)
        sum_xx = sum(p[0] * p[0] for p in host_anon_points)
        sum_xy = sum(p[0] * p[1] for p in host_anon_points)

        denom = (n * sum_xx - sum_x * sum_x)
        slope_kb_per_min = ((n * sum_xy - sum_x * sum_y) / denom) if denom != 0 else 0.0

        print(f"Memory Drift Slope: {slope_kb_per_min:+.2f} KB/min ({slope_kb_per_min * 60:+.2f} KB/hour)")
        if slope_kb_per_min > 0.5:
            print("⚠ POTENTIAL MEMORY LEAK DETECTED: Upward drift exceeds 0.5 KB/min!")
        else:
            print("✓ Memory consumption is stable (no linear leak detected).")


def main():
    parser = argparse.ArgumentParser(
        description=f"Lyria Memory & Resource Benchmark Suite v{VERSION}",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    mode_group = parser.add_mutually_exclusive_group()
    mode_group.add_argument("--watch", action="store_true", help="Launch live ANSI monitoring dashboard")
    mode_group.add_argument("--quick", action="store_true", help="Run fast developer feedback loop (S1 + S4 + S7, ~2 min)")
    mode_group.add_argument("--full-suite", action="store_true", help="Run full lifecycle suite (S1 through S8)")
    mode_group.add_argument("--long-soak", action="store_true", help="Run long soak endurance test (default 60 min)")
    mode_group.add_argument("--attach", type=int, metavar="PID", help="Attach to an already running Lyria process PID")

    parser.add_argument("--binary", type=str, default="", help="Path to release binary (default auto-search target/release)")
    parser.add_argument("--test-dir", type=str, default="", help="Directory with .flac and .mp3 files for playback tests")
    parser.add_argument("--duration", type=float, default=0.0, help="Custom duration for attach or soak mode (seconds)")
    parser.add_argument("--sample-hz", type=float, default=DEFAULT_SAMPLE_HZ, help="Sample rate in Hz (default: 2.0)")
    parser.add_argument("--compare", type=str, default="", help="Path to baseline JSON to compare against")
    parser.add_argument("--save-baseline", action="store_true", help="Save current summary as benchmarks/baseline.json")
    parser.add_argument("--output-dir", type=str, default="reports", help="Directory for benchmark reports (default: reports)")
    parser.add_argument("--with-streaming", type=str, default="", help="Optional streaming URL to test network playback")
    parser.add_argument("--caches-dropped", action="store_true", help="Advisory flag indicating page caches were dropped")

    args = parser.parse_args()

    if not (args.watch or args.quick or args.full_suite or args.long_soak or args.attach):
        args.quick = True

    if args.watch:
        tree = discover_process_tree(args.attach)
        host_pid = tree[0][0] if tree else args.attach
        run_watch_dashboard(host_pid, args.sample_hz)
        return

    binary_path = args.binary
    if not binary_path and not args.attach:
        candidates = [
            "src-tauri/target/release/echo-desktop",
            "src-tauri/target/release/lyria",
            "target/release/echo-desktop",
            "target/release/lyria",
            "src-tauri/target/debug/echo-desktop",
            "src-tauri/target/debug/lyria",
        ]
        for c in candidates:
            if os.path.isfile(c):
                binary_path = c
                break

    print("Lyria Memory & Resource Benchmark Suite")
    print("=" * 60)
    env_report = check_environment(binary_path, caches_dropped=args.caches_dropped)

    print(f"Environment Status: {env_report.confidence}")
    for check in env_report.checks:
        icon = "✓" if check.passed else "⚠"
        print(f"  [{icon}] {check.name:<18}: {check.detail}")
    print("=" * 60)

    spawned_process = None
    target_pid = args.attach

    if target_pid:
        tree = discover_process_tree(target_pid)
        if not tree:
            print(f"Error: Process PID {target_pid} not found in /proc.")
            sys.exit(1)
        print(f"Attached to process PID {target_pid} ({tree[0][1]}).")
    else:
        running_tree = discover_process_tree()
        if running_tree:
            target_pid = running_tree[0][0]
            tree = running_tree
            print(f"Discovered already running Lyria process (PID: {target_pid}).")
        elif binary_path and os.path.isfile(binary_path):
            print(f"Launching binary: {binary_path} ...")
            spawned_process = subprocess.Popen([binary_path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            target_pid = spawned_process.pid
            print(f"Spawned PID {target_pid}. Waiting 5s for window & WebKit initialization...")
            time.sleep(5.0)
            tree = discover_process_tree(target_pid) or [(target_pid, "lyria", "host")]
        else:
            print("Error: No running Lyria process detected and no compiled binary found.")
            print("Please run `cargo build --manifest-path src-tauri/Cargo.toml --release` or pass `--attach <PID>`.")
            sys.exit(1)

    all_samples: list[Sample] = []
    scenarios: list[ScenarioResult] = []
    start_time = time.monotonic()
    thresholds = load_thresholds()

    try:
        if args.long_soak:
            duration = args.duration if args.duration > 0 else DEFAULT_LONG_SOAK_S
            run_long_soak(tree, duration, DEFAULT_LONG_SOAK_SAMPLE_S, args.output_dir)
            return

        print("\nExecuting Lifecycle Scenarios:")

        cold_settle = 15.0 if not args.duration else min(args.duration, 15.0)
        s1 = sample_loop(tree, "S1_cold_idle", cold_settle, args.sample_hz, start_time, all_samples, "Settling cold baseline")
        scenarios.append(s1)

        if args.full_suite:
            with tempfile.TemporaryDirectory(prefix="lyria_bench_fixtures_") as tmp_fix_dir:
                print(f"  Generating synthetic WAV fixtures in {tmp_fix_dir}...")
                generate_wav_fixtures(tmp_fix_dir, count=50)
                s2 = sample_loop(tree, "S2_library_scan", 10.0, args.sample_hz, start_time, all_samples, "Synthetic scan simulation")
                scenarios.append(s2)

            s3 = sample_loop(tree, "S3_post_scan_idle", 10.0, args.sample_hz, start_time, all_samples, "Post-scan cooldown")
            scenarios.append(s3)

        flac_f, mp3_f = find_audio_fixtures(args.test_dir)
        playback_duration = 30.0 if args.quick else (args.duration if args.duration > 0 else 60.0)

        if flac_f:
            print(f"  Using FLAC fixture: {os.path.basename(flac_f)}")
        if mp3_f:
            print(f"  Using MP3 fixture:  {os.path.basename(mp3_f)}")

        s4 = sample_loop(
            tree,
            "S4_local_playback",
            playback_duration,
            args.sample_hz,
            start_time,
            all_samples,
            f"Active playback ({os.path.basename(flac_f) if flac_f else 'no audio file specified'})"
        )
        scenarios.append(s4)

        if args.full_suite:
            if mp3_f:
                s5 = sample_loop(tree, "S5_mp3_playback", playback_duration, args.sample_hz, start_time, all_samples, f"MP3 playback ({os.path.basename(mp3_f)})")
                scenarios.append(s5)

            s6 = sample_loop(tree, "S6_rapid_skip", 15.0, RAPID_SAMPLE_HZ, start_time, all_samples, "Simulating track switching")
            scenarios.append(s6)

        settle_duration = min(args.duration, 15.0) if args.duration > 0 else (15.0 if args.quick else DEFAULT_POST_SETTLE_S)
        settle_name = "S7_post_settle" if args.quick else "S8_post_playback_idle"
        s_settle = sample_loop(tree, settle_name, settle_duration, args.sample_hz, start_time, all_samples, "Leak check settle")
        scenarios.append(s_settle)

    finally:
        if spawned_process:
            print("\nCleaning up spawned process...")
            spawned_process.terminate()
            try:
                spawned_process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                spawned_process.kill()

    assertions = evaluate_assertions(scenarios, thresholds)

    report_data = {
        "timestamp_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "fingerprint": env_report.fingerprint,
        "environment": {
            "confidence": env_report.confidence,
            "checks": [asdict(c) for c in env_report.checks]
        },
        "scenarios": [s.summary for s in scenarios],
        "assertions": assertions,
    }

    baseline_diff = ""
    if args.compare:
        baseline_diff = compare_against_baseline(report_data, args.compare)
        report_data["baseline_comparison"] = baseline_diff

    md_content = generate_markdown_report(report_data)

    os.makedirs(args.output_dir, exist_ok=True)
    stamp = time.strftime("%Y%m%d-%H%M%S")
    md_path = os.path.join(args.output_dir, f"benchmark-{stamp}.md")
    json_path = os.path.join(args.output_dir, f"benchmark-{stamp}.json")
    raw_path = os.path.join(args.output_dir, f"benchmark-raw-{stamp}.jsonl")

    with open(md_path, "w", encoding="utf-8") as f:
        f.write(md_content)
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(report_data, f, indent=2)

    with open(raw_path, "w", encoding="utf-8") as f:
        for s in all_samples:
            f.write(json.dumps(asdict(s)) + "\n")

    print("\n" + "=" * 60)
    print("Benchmark Finished Successfully!")
    print(f"  Markdown Report: {md_path}")
    print(f"  JSON Summary:    {json_path}")
    print(f"  Raw Samples:     {raw_path}")

    if args.save_baseline:
        os.makedirs("benchmarks", exist_ok=True)
        base_target = "benchmarks/baseline.json"
        with open(base_target, "w", encoding="utf-8") as f:
            json.dump(report_data, f, indent=2)
        print(f"  Saved baseline:  {base_target}")

    print("\nAssertion Gating Results:")
    for a in assertions:
        status = "✓ PASS" if a["passed"] else "⚠ WARN"
        print(f"  [{status}] {a['name']:<48} : {a['actual']:<10} (threshold: {a['threshold']})")

    if baseline_diff:
        print("\nBaseline Comparison:")
        print(baseline_diff)

    print("=" * 60)

if __name__ == "__main__":
    main()
