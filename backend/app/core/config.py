import os
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    BACKEND_HOST: str = "127.0.0.1"
    BACKEND_PORT: int = 8000
    RUST_BINARY_PATH: str = os.getenv(
        "RUST_BINARY_PATH",
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../rust-mcp-client/target/debug/rust-mcp-client.exe"))
    )
    RUST_WORKING_DIR: str = os.getenv(
        "RUST_WORKING_DIR",
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../rust-mcp-client"))
    )
    RUST_REQUEST_TIMEOUT_SECONDS: float = 30.0
    CORS_ORIGINS: list[str] = [
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "*"
    ]

    class Config:
        env_file = ".env"
        extra = "ignore"

settings = Settings()
