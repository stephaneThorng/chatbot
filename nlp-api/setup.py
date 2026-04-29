from pathlib import Path

from setuptools import find_packages, setup


README = Path(__file__).with_name("README.md").read_text(encoding="utf-8")


setup(
    name="nlp-api",
    version="0.1.0",
    description="Multi-tenant NLP API for intent classification and NER.",
    long_description=README,
    long_description_content_type="text/markdown",
    packages=find_packages(include=["src", "src.*", "training", "training.*"]),
    include_package_data=True,
    python_requires=">=3.10",
    install_requires=[
        "fastapi==0.104.1",
        "uvicorn==0.24.0",
        "transformers==4.35.0",
        "torch==2.1.0",
        "pydantic==2.4.0",
        "pydantic-settings==2.0.0",
        "python-dotenv==1.0.0",
        "datasets==2.14.0",
        "aiofiles==23.2.1",
        "python-json-logger==2.0.7",
        "PyYAML==6.0.1",
    ],
)
