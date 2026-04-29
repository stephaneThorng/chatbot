"""Basic health check script."""

from __future__ import annotations

import argparse

import httpx


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", default="http://localhost:8000/health")
    args = parser.parse_args()

    response = httpx.get(args.url, timeout=10.0)
    response.raise_for_status()
    print(response.json())


if __name__ == "__main__":
    main()
