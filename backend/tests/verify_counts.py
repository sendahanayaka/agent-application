import urllib.request
import json

prompts = [
    "Count Senior Software Engineers",
    "Count Software Engineers",
    "Count Project Managers",
    "Count Engineering employees",
    "Count active employees",
    "Count employees on leave",
    "Count all employees"
]

header = f"{'PROMPT':<35} | {'STATUS':<10} | {'TOOL':<20} | {'ARGS':<45} | {'COUNT':<5}"
print(header)
print("-" * len(header))

for p in prompts:
    data = json.dumps({"prompt": p}).encode()
    req = urllib.request.Request("http://127.0.0.1:8000/api/chat", data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as resp:
        res = json.loads(resp.read().decode())
        status = res.get("status")
        tools = res.get("selected_tools", [])
        tool_name = tools[0] if tools else "None"
        executions = res.get("tool_executions", [])
        args = executions[0].get("arguments") if executions else {}
        count = executions[0].get("result", {}).get("count") if executions else "N/A"
        print(f"{p:<35} | {status:<10} | {tool_name:<20} | {str(args):<45} | {str(count):<5}")
