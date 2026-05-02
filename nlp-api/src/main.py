"""FastAPI application entry point."""

from __future__ import annotations

from contextlib import asynccontextmanager
import os

import uvicorn
from fastapi import FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.encoders import jsonable_encoder
from fastapi.responses import JSONResponse

from src.api.routes import router
from src.api.schemas import ErrorResponse
from src.config import settings
from src.services.nlp_service import NLPService
from src.utils.logger import configure_logging, get_logger


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize and tear down application state."""

    configure_logging()
    logger = get_logger(__name__)
    service = NLPService()
    await service.initialize()
    app.state.nlp_service = service
    logger.info("Application startup complete.")
    yield


def create_app(service: NLPService | None = None) -> FastAPI:
    """Create a FastAPI app instance."""

    app = FastAPI(
        title="NLP API",
        version="0.1.0",
        description="Multi-tenant intent classification and NER service.",
        lifespan=lifespan if service is None else None,
    )
    if service is not None:
        app.state.nlp_service = service
    app.include_router(router)

    @app.exception_handler(RequestValidationError)
    async def validation_exception_handler(
        request: Request,
        exc: RequestValidationError,
    ) -> JSONResponse:
        del request
        payload = ErrorResponse(
            error="validation_error",
            message="Request payload validation failed.",
            details={"errors": jsonable_encoder(exc.errors())},
        )
        return JSONResponse(status_code=400, content=payload.model_dump())

    return app


app = create_app()


def run() -> None:
    """Run the API server with settings-backed defaults."""

    uvicorn.run(
        "src.main:app",
        host=settings.service_host,
        port=settings.service_port,
        reload=os.getenv("UVICORN_RELOAD", "false").lower() == "true",
        workers=settings.uvicorn_workers,
    )


if __name__ == "__main__":
    run()
