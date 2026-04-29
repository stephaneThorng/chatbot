"""Download configured Hugging Face models into the local cache."""

from __future__ import annotations

import asyncio

from src.services.model_manager import ModelManager


async def main() -> None:
    manager = ModelManager()
    await manager.download_bundle()
    print("Models downloaded successfully.")


if __name__ == "__main__":
    asyncio.run(main())
