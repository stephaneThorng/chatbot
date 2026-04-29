"""API routes."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request, status

from src.api.schemas import AnalysisRequest, AnalysisResponse, ErrorResponse, HealthResponse
from src.services.nlp_service import NLPService


router = APIRouter()


def _get_service(request: Request) -> NLPService:
    service = getattr(request.app.state, "nlp_service", None)
    if service is None:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="NLP service is not initialized.",
        )
    return service


@router.post(
    "/analyze",
    response_model=AnalysisResponse,
    responses={503: {"model": ErrorResponse}},
)
async def analyze(request: Request, payload: AnalysisRequest) -> AnalysisResponse:
    """Run intent classification and entity extraction."""

    service = _get_service(request)
    try:
        return await service.analyze(
            text=payload.text,
            domain=payload.domain,
            context=payload.context,
        )
    except RuntimeError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=str(exc),
        ) from exc
    except ValueError as exc:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(exc),
        ) from exc


@router.get("/health", response_model=HealthResponse)
async def health(request: Request) -> HealthResponse:
    """Return service health details."""

    service = _get_service(request)
    return service.health()
