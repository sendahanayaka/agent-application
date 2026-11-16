from contextlib import asynccontextmanager
import logging
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from app.api.endpoints import router as api_router
from app.core.config import settings
from app.services.rust_client import rust_manager

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("FastAPIBackend")

@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Initializing FastAPI Backend lifespan...")
    try:
        await rust_manager.start()
        logger.info("Rust MCP Client service started successfully.")
    except Exception as e:
        logger.exception(f"Failed to start Rust MCP Client service on startup: {e}")
    yield
    logger.info("Shutting down FastAPI Backend lifespan...")
    await rust_manager.stop()
    logger.info("Rust MCP Client service shut down cleanly.")

app = FastAPI(
    title="SPIL Labs Agentic Employee Management API",
    description="Python FastAPI Gateway forwarding natural language queries to Rust MCP Client and .NET MCP Server.",
    version="1.0.0",
    lifespan=lifespan,
)

# CORS configuration for React frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Centralized global exception handler
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(f"Unhandled exception on {request.method} {request.url.path}: {exc}")
    return JSONResponse(
        status_code=500,
        content={"detail": "An internal server error occurred.", "error": str(exc)},
    )

app.include_router(api_router, prefix="/api")

@app.get("/", summary="Root Status")
async def root():
    return {
        "service": "SPIL Labs Agentic Employee Management Backend",
        "status": "online",
        "docs": "/docs",
    }
