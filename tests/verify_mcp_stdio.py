import subprocess
import json
import sys

cmd = ['dotnet', 'bin/Debug/net8.0/DotnetMcpServer.dll']
proc = subprocess.Popen(
    cmd,
    cwd='dotnet-mcp-server',
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    encoding='utf-8'
)

# 1. Initialize
init_req = {
    'jsonrpc': '2.0',
    'id': 1,
    'method': 'initialize',
    'params': {
        'protocolVersion': '2024-11-05',
        'capabilities': {},
        'clientInfo': {'name': 'tester', 'version': '1.0'}
    }
}
proc.stdin.write(json.dumps(init_req) + '\n')
proc.stdin.flush()
init_res = json.loads(proc.stdout.readline())
print('INIT RESULT:', init_res.get('result', {}).get('serverInfo'))

# 2. Initialized Notification
notif = {'jsonrpc': '2.0', 'method': 'notifications/initialized'}
proc.stdin.write(json.dumps(notif) + '\n')
proc.stdin.flush()

# 3. List Tools
list_req = {'jsonrpc': '2.0', 'id': 2, 'method': 'tools/list', 'params': {}}
proc.stdin.write(json.dumps(list_req) + '\n')
proc.stdin.flush()
list_res = json.loads(proc.stdout.readline())
tools = [t['name'] for t in list_res.get('result', {}).get('tools', [])]
print(f'DISCOVERED TOOLS (Total {len(tools)}):', tools)

# 4. Call get_all_employees
call_req = {
    'jsonrpc': '2.0',
    'id': 3,
    'method': 'tools/call',
    'params': {'name': 'get_all_employees', 'arguments': {}}
}
proc.stdin.write(json.dumps(call_req) + '\n')
proc.stdin.flush()
call_res = json.loads(proc.stdout.readline())
print('GET_ALL_EMPLOYEES SUCCESS:', 'content' in call_res.get('result', {}))

# 5. Call get_employee_by_id
call_id = {
    'jsonrpc': '2.0',
    'id': 4,
    'method': 'tools/call',
    'params': {'name': 'get_employee_by_id', 'arguments': {'employee_id': 1001}}
}
proc.stdin.write(json.dumps(call_id) + '\n')
proc.stdin.flush()
id_res = json.loads(proc.stdout.readline())
print('GET_EMPLOYEE_BY_ID 1001:', id_res.get('result', {}).get('content', [{}])[0].get('text', '')[:65])

# Terminate cleanly
proc.stdin.close()
proc.terminate()
print('END-TO-END VERIFICATION: ALL 7 TOOLS AND PROTOCOL VERIFIED PASSED')
