"""In-process request metrics."""

from __future__ import annotations

import statistics
import time
from dataclasses import dataclass, field
from typing import Dict, List


@dataclass(slots=True)
class RequestMetrics:
    """Per-request metric state."""

    started_at: float
    ended_at: float | None = None
    intent_name: str | None = None
    intent_confidence: float | None = None
    entity_count: int = 0
    fast_path: bool = False

    @property
    def duration_ms(self) -> float:
        if self.ended_at is None:
            return 0.0
        return (self.ended_at - self.started_at) * 1000


@dataclass(slots=True)
class AggregatedMetrics:
    """Simple aggregate metrics."""

    total_requests: int = 0
    successful_requests: int = 0
    failed_requests: int = 0
    durations_ms: List[float] = field(default_factory=list)
    intents: Dict[str, int] = field(default_factory=dict)

    def snapshot(self) -> Dict[str, float | int | Dict[str, int]]:
        sorted_durations = sorted(self.durations_ms)
        p95_index = min(int(len(sorted_durations) * 0.95), max(len(sorted_durations) - 1, 0))
        return {
            "total_requests": self.total_requests,
            "successful_requests": self.successful_requests,
            "failed_requests": self.failed_requests,
            "avg_latency_ms": round(statistics.fmean(sorted_durations), 3) if sorted_durations else 0.0,
            "p95_latency_ms": round(sorted_durations[p95_index], 3) if sorted_durations else 0.0,
            "intent_counts": dict(self.intents),
        }


class MetricsCollector:
    """Collect metrics for the process lifetime."""

    def __init__(self) -> None:
        self.aggregated = AggregatedMetrics()

    def track_request(self) -> RequestMetrics:
        return RequestMetrics(started_at=time.perf_counter())

    def finalize_request(self, request: RequestMetrics, success: bool) -> None:
        request.ended_at = time.perf_counter()
        self.aggregated.total_requests += 1
        self.aggregated.durations_ms.append(request.duration_ms)
        if success:
            self.aggregated.successful_requests += 1
        else:
            self.aggregated.failed_requests += 1
        if request.intent_name:
            self.aggregated.intents[request.intent_name] = self.aggregated.intents.get(request.intent_name, 0) + 1


metrics_collector = MetricsCollector()
