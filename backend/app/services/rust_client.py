import asyncio
import json
import logging
import os
import subprocess
from typing import Any, Optional
from app.core.config import settings

logger = logging.getLogger("RustClientProcess")

class RustProcessManager:
    """
    Manages a persistent child process running the Rust MCP Client binary.
    Communicates via newline-delimited JSON over STDIN / STDOUT.
    """
    def __init__(self):
        self._process: Optional[asyncio.subprocess.Process] = None
        self._request_counter: int = 0
        self._pending_requests: dict[int, asyncio.Future] = {}
        self._lock = asyncio.Lock()
        self._reader_task: Optional[asyncio.Task] = None

    async def start(self):
        if self._process is not None and self._process.returncode is None:
            return

        rust_exe = settings.RUST_BINARY_PATH
        if not os.path.exists(rust_exe):
            raise FileNotFoundError(f"Rust binary not found at: {rust_exe}")

        logger.info(f"Starting persistent Rust MCP Client process: {rust_exe}")
        self._process = await asyncio.create_subprocess_exec(
            rust_exe,
            cwd=settings.RUST_WORKING_DIR,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )

        # Background reader for STDOUT messages
        self._reader_task = asyncio.create_task(self._read_stdout())
        # Background reader for STDERR diagnostic logs
        asyncio.create_task(self._read_stderr())
        logger.info(f"Rust MCP Client process started with PID: {self._process.pid}")

    async def _read_stdout(self):
        try:
            while self._process and self._process.stdout and not self._process.stdout.at_eof():
                line = await self._process.stdout.readline()
                if not line:
                    break
                line_str = line.decode("utf-8").strip()
                if not line_str:
                    continue
                try:
                    msg = json.loads(line_str)
                    req_id = msg.get("id")
                    if req_id is not None and req_id in self._pending_requests:
                        future = self._pending_requests.pop(req_id)
                        if not future.done():
                            future.set_result(msg)
                except Exception as e:
                    logger.error(f"Error parsing STDOUT JSON line: {e} | Line: {line_str}")
        except asyncio.CancelledError:
            pass
        except Exception as e:
            logger.error(f"Error in Rust STDOUT reader loop: {e}")

    async def _read_stderr(self):
        try:
            while self._process and self._process.stderr and not self._process.stderr.at_eof():
                line = await self._process.stderr.readline()
                if not line:
                    break
                logger.info(f"[Rust STDERR] {line.decode('utf-8').strip()}")
        except Exception:
            pass

    async def send_command(self, action: str, prompt: Optional[str] = None, token: Optional[str] = None) -> dict[str, Any]:
        async with self._lock:
            if self._process is None or self._process.returncode is not None:
                logger.warning("Rust process is not running. Attempting restart...")
                await self.start()

            self._request_counter += 1
            req_id = self._request_counter

            future = asyncio.get_running_loop().create_future()
            self._pending_requests[req_id] = future

            payload = {
                "id": req_id,
                "action": action,
                "prompt": prompt,
                "token": token
            }
            json_line = json.dumps(payload) + "\n"

            if self._process.stdin is None:
                raise RuntimeError("Rust child process STDIN pipe is not available")

            self._process.stdin.write(json_line.encode("utf-8"))
            await self._process.stdin.drain()

        # Await response with timeout
        try:
            response = await asyncio.wait_for(future, timeout=settings.RUST_REQUEST_TIMEOUT_SECONDS)
            if not response.get("success", False):
                raise RuntimeError(response.get("error", "Unknown error returned from Rust client"))
            return response.get("data", {})
        except asyncio.TimeoutError:
            self._pending_requests.pop(req_id, None)
            raise TimeoutError(f"Rust MCP client timed out after {settings.RUST_REQUEST_TIMEOUT_SECONDS}s")

    async def stop(self):
        if self._reader_task:
            self._reader_task.cancel()
        if self._process:
            try:
                self._process.terminate()
                await asyncio.wait_for(self._process.wait(), timeout=3.0)
            except Exception:
                if self._process:
                    self._process.kill()
            self._process = None
        logger.info("Rust MCP Client process terminated cleanly.")

    def is_alive(self) -> bool:
        return self._process is not None and self._process.returncode is None

rust_manager = RustProcessManager()
