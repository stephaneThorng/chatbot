"""Simple HTTP benchmark for the analyze endpoint."""

from __future__ import annotations

import argparse
import statistics
import time

import httpx


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", default="http://localhost:8000/analyze")
    parser.add_argument("--iterations", type=int, default=25)
    args = parser.parse_args()

    payload = {
        "text": "Je souhaite reserv pour 4 personnes demain a 19h",
        "domain": "restaurant",
        "context": None,
    }
    durations = []
    with httpx.Client(timeout=10.0) as client:
        for _ in range(args.iterations):
            started = time.perf_counter()
            response = client.post(args.url, json=payload)
            response.raise_for_status()
            durations.append((time.perf_counter() - started) * 1000)
    print(
        {
            "iterations": args.iterations,
            "avg_ms": round(statistics.fmean(durations), 3),
            "min_ms": round(min(durations), 3),
            "max_ms": round(max(durations), 3),
        }
    )


if __name__ == "__main__":
    main()
