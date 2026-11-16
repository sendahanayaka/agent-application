from typing import Any, Optional
from pydantic import BaseModel, Field

class ChatRequest(BaseModel):
    prompt: str = Field(..., min_length=1, description="Natural language prompt entered by user")

class ConfirmRequest(BaseModel):
    token: str = Field(..., min_length=3, description="Confirmation token for a pending write operation (e.g. CONF-123456)")

class SingleToolExecution(BaseModel):
    tool_name: str
    arguments: dict[str, Any] = Field(default_factory=dict)
    success: bool
    result: Optional[Any] = None
    error: Optional[str] = None

class PendingOperation(BaseModel):
    confirmation_token: str
    tool_name: str
    arguments: dict[str, Any]
    summary: str
    created_at_unix: int

class PipelineResponseModel(BaseModel):
    status: str
    prompt: str
    detected_intents: list[str] = Field(default_factory=list)
    selected_tools: list[str] = Field(default_factory=list)
    message: str
    pending_operation: Optional[PendingOperation] = None
    tool_executions: list[SingleToolExecution] = Field(default_factory=list)
    missing_parameters: Optional[list[str]] = None
    ambiguous_options: Optional[list[str]] = None

class HealthResponse(BaseModel):
    status: str
    rust_process: str
    mcp_server: Optional[dict[str, Any]] = None
    tools_count: int

class ToolPropertySchema(BaseModel):
    prop_type: Optional[str] = Field(None, alias="type")
    description: Optional[str] = None
    default: Optional[Any] = None

class ToolInputSchema(BaseModel):
    schema_type: str = Field("object", alias="type")
    properties: dict[str, Any] = Field(default_factory=dict)
    required: list[str] = Field(default_factory=list)

class ToolInfo(BaseModel):
    name: str
    title: Optional[str] = None
    description: Optional[str] = None
    input_schema: ToolInputSchema = Field(..., alias="inputSchema")
    annotations: Optional[dict[str, Any]] = None

class ToolsResponse(BaseModel):
    count: int
    tools: list[Any]
