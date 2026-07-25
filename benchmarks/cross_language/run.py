#!/usr/bin/env python3
"""Reproducible cross-language end-to-end benchmark runner for Hüma."""

from __future__ import annotations

import argparse
import json
import math
import os
import platform
import random
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple


ROOT = Path(__file__).resolve().parents[2]
SUITE = Path(__file__).resolve().parent
PROGRAMS = SUITE / "programs"
BUILD = SUITE / ".build"
TIMEOUT_SECONDS = 30
RANDOM_SEED = 0x48554D41

WORKLOADS: Dict[str, Dict[str, object]] = {
    "branch_loop": {
        "expected": "99740874156250",
        "description": "200.000 adımlı koşullu, sonlu güvenli-tamsayı durum geçişi",
    },
    "numeric_loop": {
        "expected": "429301206370208",
        "description": "200.000 adımlı güvenli-tamsayı LCG ve modüler toplam",
    },
    "function_loop": {
        "expected": "2090659698",
        "description": "100.000 kullanıcı fonksiyonu çağrılı modüler durum geçişi",
    },
    "collection_loop": {
        "expected": "24836475",
        "description": "50.000 öğe ekleme, indeksleme ve toplama",
    },
}


@dataclass(frozen=True)
class Candidate:
    candidate_id: str
    label: str
    toolchain: str
    workload: str
    command: Tuple[str, ...]


def run_command(
    command: Sequence[str],
    *,
    check: bool = True,
    timeout: int = TIMEOUT_SECONDS,
    env: Optional[Dict[str, str]] = None,
) -> subprocess.CompletedProcess:
    merged_env = os.environ.copy()
    merged_env.update({"LC_ALL": "C", "LANG": "C"})
    if env:
        merged_env.update(env)
    return subprocess.run(
        list(command),
        cwd=ROOT,
        env=merged_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        timeout=timeout,
        check=check,
    )


def tool_version(command: Sequence[str], stderr: bool = False) -> Optional[str]:
    try:
        completed = run_command(command)
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return None
    text = completed.stderr if stderr else completed.stdout
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    return lines[0] if lines else None


def compile_programs() -> Tuple[List[Candidate], Dict[str, str], List[str]]:
    BUILD.mkdir(parents=True, exist_ok=True)
    candidates: List[Candidate] = []
    versions: Dict[str, str] = {}
    skipped: List[str] = []

    huma = ROOT / "target" / "release" / "huma"
    run_command(["cargo", "build", "--release", "-p", "huma-cli"], timeout=600)
    versions["Hüma"] = tool_version([str(huma), "--version"]) or "bilinmiyor"

    for workload in WORKLOADS:
        source = PROGRAMS / workload / "huma.hb"
        interpreter = Candidate(
            f"huma-interpreter:{workload}",
            "Hüma yorumlayıcı",
            "Hüma",
            workload,
            (str(huma), "run", str(source)),
        )
        interpreter_error = probe_candidate(interpreter)
        if interpreter_error:
            raise RuntimeError(
                f"Kanonik Hüma yorumlayıcı {workload} iş yükünü doğrulamadı: "
                f"{interpreter_error}"
            )
        candidates.append(interpreter)

        vm = Candidate(
            f"huma-vm:{workload}",
            "Hüma VM",
            "Hüma",
            workload,
            (str(huma), "run", str(source), "--vm"),
        )
        vm_error = probe_candidate(vm)
        if vm_error:
            skipped.append(f"Hüma VM/{workload}: {vm_error}")
        else:
            candidates.append(vm)

        aot_output = BUILD / f"huma-aot-{workload}"
        aot = run_command(
            [str(huma), "aot", str(source), "-o", str(aot_output)],
            check=False,
            timeout=120,
        )
        if aot.returncode == 0:
            aot_candidate = Candidate(
                f"huma-aot:{workload}",
                "Hüma AOT",
                "Hüma",
                workload,
                (str(aot_output),),
            )
            aot_error = probe_candidate(aot_candidate)
            if aot_error:
                skipped.append(f"Hüma AOT/{workload}: {aot_error}")
            else:
                candidates.append(aot_candidate)
        else:
            skipped.append(
                f"Hüma AOT/{workload}: desteklenmeyen alt küme "
                f"({last_error_line(aot.stderr)})"
            )

    interpreted = [
        (
            "python",
            "Python",
            "python3",
            ["python3", "--version"],
            False,
            "python.py",
        ),
        ("node", "Node.js", "node", ["node", "--version"], False, "node.js"),
        ("ruby", "Ruby", "ruby", ["ruby", "--version"], False, "ruby.rb"),
    ]
    for candidate_id, label, executable, version_cmd, version_stderr, filename in interpreted:
        path = shutil.which(executable)
        if not path:
            skipped.append(f"{label}: araç zinciri bulunamadı")
            continue
        versions[label] = tool_version(version_cmd, stderr=version_stderr) or "bilinmiyor"
        for workload in WORKLOADS:
            candidates.append(
                Candidate(
                    f"{candidate_id}:{workload}",
                    label,
                    label,
                    workload,
                    (path, str(PROGRAMS / workload / filename)),
                )
            )

    clang = shutil.which("clang")
    if clang:
        versions["C/Clang"] = tool_version([clang, "--version"]) or "bilinmiyor"
        for workload in WORKLOADS:
            output = BUILD / f"c-{workload}"
            run_command(
                [
                    clang,
                    "-O3",
                    "-std=c17",
                    str(PROGRAMS / workload / "program.c"),
                    "-o",
                    str(output),
                ],
                timeout=120,
            )
            candidates.append(
                Candidate(
                    f"c:{workload}",
                    "C -O3",
                    "C/Clang",
                    workload,
                    (str(output),),
                )
            )
    else:
        skipped.append("C/Clang: araç zinciri bulunamadı")

    rustc = shutil.which("rustc")
    if rustc:
        versions["Rust"] = tool_version([rustc, "--version"]) or "bilinmiyor"
        for workload in WORKLOADS:
            output = BUILD / f"rust-{workload}"
            run_command(
                [
                    rustc,
                    "-C",
                    "opt-level=3",
                    "-C",
                    "debuginfo=0",
                    str(PROGRAMS / workload / "program.rs"),
                    "-o",
                    str(output),
                ],
                timeout=180,
            )
            candidates.append(
                Candidate(
                    f"rust:{workload}",
                    "Rust -O",
                    "Rust",
                    workload,
                    (str(output),),
                )
            )
    else:
        skipped.append("Rust: araç zinciri bulunamadı")

    swiftc = shutil.which("swiftc")
    if swiftc:
        versions["Swift"] = tool_version(["swift", "--version"]) or "bilinmiyor"
        for workload in WORKLOADS:
            output = BUILD / f"swift-{workload}"
            run_command(
                [
                    swiftc,
                    "-O",
                    str(PROGRAMS / workload / "program.swift"),
                    "-o",
                    str(output),
                ],
                timeout=180,
            )
            candidates.append(
                Candidate(
                    f"swift:{workload}",
                    "Swift -O",
                    "Swift",
                    workload,
                    (str(output),),
                )
            )
    else:
        skipped.append("Swift: araç zinciri bulunamadı")

    javac = shutil.which("javac")
    java = shutil.which("java")
    if javac and java:
        versions["Java"] = tool_version([java, "-version"], stderr=True) or "bilinmiyor"
        java_build = BUILD / "java"
        java_build.mkdir(parents=True, exist_ok=True)
        for workload in WORKLOADS:
            source = PROGRAMS / workload / JAVA_SOURCES[workload]
            run_command([javac, "-d", str(java_build), str(source)], timeout=180)
            candidates.append(
                Candidate(
                    f"java:{workload}",
                    "Java",
                    "Java",
                    workload,
                    (java, "-cp", str(java_build), JAVA_CLASSES[workload]),
                )
            )
    else:
        skipped.append("Java: java/javac araç zinciri bulunamadı")

    return candidates, versions, skipped


JAVA_SOURCES = {
    "branch_loop": "BranchLoop.java",
    "numeric_loop": "NumericLoop.java",
    "function_loop": "FunctionLoop.java",
    "collection_loop": "CollectionLoop.java",
}

JAVA_CLASSES = {
    "branch_loop": "BranchLoop",
    "numeric_loop": "NumericLoop",
    "function_loop": "FunctionLoop",
    "collection_loop": "CollectionLoop",
}


def last_error_line(stderr: str) -> str:
    lines = [line.strip() for line in stderr.splitlines() if line.strip()]
    return lines[-1] if lines else "ayrıntı yok"


def probe_candidate(candidate: Candidate) -> Optional[str]:
    completed = run_command(candidate.command, check=False)
    if completed.returncode != 0:
        detail = last_error_line(completed.stderr)
        return f"çıkış {completed.returncode}: {detail}"
    expected = str(WORKLOADS[candidate.workload]["expected"])
    actual = completed.stdout.strip()
    if actual != expected:
        return f"yanlış çıktı: beklenen={expected!r}, gelen={actual!r}"
    return None


def validate_output(candidate: Candidate, stdout: str) -> None:
    expected = str(WORKLOADS[candidate.workload]["expected"])
    actual = stdout.strip()
    if actual != expected:
        raise RuntimeError(
            f"{candidate.candidate_id} yanlış çıktı üretti: "
            f"beklenen={expected!r}, gelen={actual!r}"
        )


def execute_timed(candidate: Candidate) -> float:
    start = time.perf_counter_ns()
    completed = run_command(candidate.command)
    elapsed_ms = (time.perf_counter_ns() - start) / 1_000_000.0
    validate_output(candidate, completed.stdout)
    return elapsed_ms


def percentile(values: Sequence[float], fraction: float) -> float:
    ordered = sorted(values)
    index = max(0, math.ceil(len(ordered) * fraction) - 1)
    return ordered[index]


def measure_rss(candidate: Candidate) -> Optional[int]:
    time_binary = Path("/usr/bin/time")
    if not time_binary.exists():
        return None
    if sys.platform == "darwin":
        command = [str(time_binary), "-l", *candidate.command]
        marker = "maximum resident set size"
        multiplier = 1
    elif sys.platform.startswith("linux"):
        command = [str(time_binary), "-v", *candidate.command]
        marker = "Maximum resident set size (kbytes)"
        multiplier = 1024
    else:
        return None
    completed = run_command(command)
    validate_output(candidate, completed.stdout)
    for line in completed.stderr.splitlines():
        if marker in line:
            if ":" in line:
                raw = line.split(":", 1)[1].strip().split()[0]
            else:
                raw = line.strip().split()[0]
            return int(raw) * multiplier
    return None


def machine_info() -> Dict[str, object]:
    cpu = platform.processor() or platform.machine()
    memory_bytes: Optional[int] = None
    if sys.platform == "darwin":
        cpu_result = tool_version(["sysctl", "-n", "machdep.cpu.brand_string"])
        memory_result = tool_version(["sysctl", "-n", "hw.memsize"])
        cpu = cpu_result or cpu
        if memory_result and memory_result.isdigit():
            memory_bytes = int(memory_result)
    git_revision = tool_version(["git", "rev-parse", "HEAD"])
    return {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "cpu": cpu,
        "memory_bytes": memory_bytes,
        "python_runner": platform.python_version(),
        "git_revision": git_revision,
    }


def benchmark(
    candidates: Sequence[Candidate],
    samples: int,
    warmups: int,
    memory_samples: int,
) -> List[Dict[str, object]]:
    for candidate in candidates:
        for _ in range(warmups):
            execute_timed(candidate)

    timings: Dict[str, List[float]] = {
        candidate.candidate_id: [] for candidate in candidates
    }
    rng = random.Random(RANDOM_SEED)
    for _ in range(samples):
        ordered = list(candidates)
        rng.shuffle(ordered)
        for candidate in ordered:
            timings[candidate.candidate_id].append(execute_timed(candidate))

    results: List[Dict[str, object]] = []
    for candidate in candidates:
        values = timings[candidate.candidate_id]
        rss_values = [
            rss
            for rss in (measure_rss(candidate) for _ in range(memory_samples))
            if rss is not None
        ]
        results.append(
            {
                "id": candidate.candidate_id,
                "label": candidate.label,
                "toolchain": candidate.toolchain,
                "workload": candidate.workload,
                "command": list(candidate.command),
                "samples": len(values),
                "median_ms": statistics.median(values),
                "mean_ms": statistics.mean(values),
                "min_ms": min(values),
                "p95_ms": percentile(values, 0.95),
                "stddev_ms": statistics.stdev(values) if len(values) > 1 else 0.0,
                "median_rss_bytes": (
                    int(statistics.median(rss_values)) if rss_values else None
                ),
                "raw_ms": values,
            }
        )
    return results


def render_markdown(document: Dict[str, object]) -> str:
    machine = document["machine"]
    lines = [
        "# Diller arası benchmark sonucu",
        "",
        f"- Tarih (UTC): `{document['generated_at_utc']}`",
        f"- Git: `{machine['git_revision']}`",
        f"- Sistem: `{machine['platform']}`",
        f"- CPU: `{machine['cpu']}`",
        f"- Örnek/ısınma: `{document['samples']}` / `{document['warmups']}`",
        "",
        "Süreler süreç başlangıcı dâhil medyandır. RSS ayrı süreçlerde ölçülen "
        "medyandır. Küçük değerlerde işletim sistemi zamanlayıcısı baskındır.",
        "",
    ]
    results = document["results"]
    for workload, metadata in WORKLOADS.items():
        lines.extend(
            [
                f"## {workload}",
                "",
                str(metadata["description"]),
                "",
                "| Yürütme | Medyan (ms) | Ortalama (ms) | p95 (ms) | RSS (MiB) |",
                "|---|---:|---:|---:|---:|",
            ]
        )
        workload_results = [item for item in results if item["workload"] == workload]
        workload_results.sort(key=lambda item: item["median_ms"])
        for item in workload_results:
            rss = item["median_rss_bytes"]
            rss_text = "—" if rss is None else f"{rss / 1024 / 1024:.2f}"
            lines.append(
                f"| {item['label']} | {item['median_ms']:.3f} | "
                f"{item['mean_ms']:.3f} | {item['p95_ms']:.3f} | {rss_text} |"
            )
        lines.append("")

    lines.extend(["## Araç sürümleri", ""])
    for name, version in document["tool_versions"].items():
        lines.append(f"- {name}: `{version}`")
    if document["skipped"]:
        lines.extend(["", "## Atlananlar", ""])
        lines.extend(f"- {item}" for item in document["skipped"])
    lines.append("")
    return "\n".join(lines)


def write_output(path: Optional[str], content: str) -> None:
    if not path:
        return
    output = Path(path)
    if not output.is_absolute():
        output = ROOT / output
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_name(f".{output.name}.tmp-{os.getpid()}")
    temporary.write_text(content, encoding="utf-8")
    os.replace(temporary, output)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--samples", type=int, default=30)
    parser.add_argument("--warmups", type=int, default=5)
    parser.add_argument("--memory-samples", type=int, default=3)
    parser.add_argument("--json-output")
    parser.add_argument("--markdown-output")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.samples < 5 or args.warmups < 1 or args.memory_samples < 0:
        raise SystemExit("samples >= 5, warmups >= 1 ve memory-samples >= 0 olmalı")

    candidates, versions, skipped = compile_programs()
    for candidate in candidates:
        validate_output(candidate, run_command(candidate.command).stdout)

    results = benchmark(
        candidates,
        samples=args.samples,
        warmups=args.warmups,
        memory_samples=args.memory_samples,
    )
    document: Dict[str, object] = {
        "schema_version": 1,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "samples": args.samples,
        "warmups": args.warmups,
        "memory_samples": args.memory_samples,
        "random_seed": RANDOM_SEED,
        "machine": machine_info(),
        "tool_versions": versions,
        "workloads": WORKLOADS,
        "skipped": skipped,
        "results": results,
    }
    json_text = json.dumps(document, ensure_ascii=False, indent=2) + "\n"
    markdown_text = render_markdown(document)
    write_output(args.json_output, json_text)
    write_output(args.markdown_output, markdown_text)
    print(markdown_text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
