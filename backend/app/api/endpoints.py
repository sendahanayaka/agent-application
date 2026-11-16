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

@router.post(
    "/chat",
    response_model=PipelineResponseModel,
    summary="Submit Natural Language Prompt",
    description="Forwards a natural language employee-management prompt to Rust for deterministic intent matching, validation, and tool execution.",
)
async def chat(request: ChatRequest):
    try:
        data = await rust_manager.send_command(action="chat", prompt=request.prompt)
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
    try:
        data = await rust_manager.send_command(action="confirm", token=request.token)
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
