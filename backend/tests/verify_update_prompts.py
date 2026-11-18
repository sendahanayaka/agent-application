import urllib.request
import json

prompts = [
    "update employee id 1038 to 1021 and add phone number as 0771231234",
    "Update employee ID 1038 phone number to 0771231234",
    "Change phone number of employee 1038 to 0771231234",
    "Update employee 1038 without specifying any update field",
    "Update employee ID 1038 to 1021"
]

for p in prompts:
    data = json.dumps({"prompt": p}).encode()
    req = urllib.request.Request("http://127.0.0.1:8000/api/chat", data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as resp:
        res = json.loads(resp.read().decode())
        print(f"PROMPT:  {p}")
        print(f"STATUS:  {res.get('status')}")
        print(f"MESSAGE: {res.get('message')}")
        if res.get("pending_operation"):
            print(f"ARGS:    {res.get('pending_operation')['arguments']}")
            print(f"SUMMARY: {res.get('pending_operation')['summary']}")
        print("-" * 80)
