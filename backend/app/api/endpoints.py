import time
from typing import Any
from fastapi import APIRouter, HTTPException, status
from app.models.schemas import (
    ChatRequest,
    ConfirmRequest,
    HealthResponse,
    PipelineResponseModel,
    ToolsResponse,
)
from app.services.rust_client import rust_manager

router = APIRouter()

# In-memory session store mapping conversation_id -> {"context": dict, "updated_at": float}
_SESSION_STORE: dict[str, dict[str, Any]] = {}
_SESSION_TTL_SECONDS = 3600  # 1 hour expiration

def _cleanup_expired_sessions():
    now = time.time()
    expired = [cid for cid, s in _SESSION_STORE.items() if now - s.get("updated_at", 0) > _SESSION_TTL_SECONDS]
    for cid in expired:
        _SESSION_STORE.pop(cid, None)

@router.post(
    "/chat",
    response_model=PipelineResponseModel,
    summary="Submit Natural Language Prompt",
    description="Forwards a natural language employee-management prompt to Rust for deterministic intent matching, validation, and tool execution.",
)
async def chat(request: ChatRequest):
    _cleanup_expired_sessions()
    cid = request.conversation_id or "default"
    session_data = _SESSION_STORE.get(cid, {})
    current_context = session_data.get("context")

    try:
        data = await rust_manager.send_command(
            action="chat",
            prompt=request.prompt,
            context=current_context,
        )
        # If response returned updated conversation context, persist it for this session
        if data.get("conversation_context"):
            _SESSION_STORE[cid] = {
                "context": data["conversation_context"],
                "updated_at": time.time(),
            }
        return PipelineResponseModel(**data)
    except TimeoutError as e:
        raise HTTPException(
            status_code=status.HTTP_504_GATEWAY_TIMEOUT,
            detail=str(e),
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"Rust MCP client error: {e}",
        )

@router.post(
    "/confirm",
    response_model=PipelineResponseModel,
    summary="Confirm Write Operation",
    description="Submits a confirmation token to execute a pending write operation (add_employee or update_employee).",
)
async def confirm(request: ConfirmRequest):
    cid = request.conversation_id or "default"
    session_data = _SESSION_STORE.get(cid, {})
    current_context = session_data.get("context")

    try:
        data = await rust_manager.send_command(
            action="confirm",
            token=request.token,
            context=current_context,
        )
        if data.get("conversation_context"):
            _SESSION_STORE[cid] = {
                "context": data["conversation_context"],
                "updated_at": time.time(),
            }
        return PipelineResponseModel(**data)
    except TimeoutError as e:
        raise HTTPException(
            status_code=status.HTTP_504_GATEWAY_TIMEOUT,
            detail=str(e),
        )
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"Confirmation execution error: {e}",
        )

@router.delete(
    "/conversations/{conversation_id}",
    summary="Clear Conversation Session Context",
    description="Clears session conversational memory and associated employee references.",
)
async def clear_conversation(conversation_id: str):
    _SESSION_STORE.pop(conversation_id, None)
    return {"status": "cleared", "conversation_id": conversation_id}

@router.get(
    "/tools",
    response_model=ToolsResponse,
    summary="Get Discovered MCP Tools",
    description="Returns the live list of tools and JSON input schemas discovered from the .NET MCP Server.",
)
async def get_tools():
    try:
        data = await rust_manager.send_command(action="tools")
        if isinstance(data, list):
            return ToolsResponse(count=len(data), tools=data)
        return ToolsResponse(count=0, tools=[])
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"Failed to retrieve tools: {e}",
        )

@router.get(
    "/health",
    response_model=HealthResponse,
    summary="Health Check",
    description="Checks the operational health of FastAPI, Rust persistent process, and the underlying .NET MCP Server.",
)
async def health():
    try:
        data = await rust_manager.send_command(action="health")
        return HealthResponse(
            status="healthy",
            rust_process="running" if rust_manager.is_alive() else "stopped",
            mcp_server=data.get("mcp_server"),
            tools_count=data.get("tools_count", 0),
        )
    except Exception as e:
        return HealthResponse(
            status="degraded",
            rust_process="error",
            mcp_server={"error": str(e)},
            tools_count=0,
        )

@router.get(
    "/conversations",
    summary="Get Conversations (Session Stub)",
    description="Session history placeholder endpoint for frontend chat persistence.",
)
async def get_conversations():
    return {"conversations": []}
