import asyncio
import subprocess
from httpx import AsyncClient, ASGITransport
from app.main import app
from app.services.rust_client import rust_manager

def test_api_full_suite():
    async def async_suite():
        await rust_manager.start()
        try:
            async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as ac:
                # 1. Health
                print("1. Testing /api/health...")
                r_health = await ac.get("/api/health")
                assert r_health.status_code == 200, f"Health failed: {r_health.text}"
                assert r_health.json()["status"] == "healthy"
                assert r_health.json()["tools_count"] == 7
                print("   PASS: /api/health")

                # 2. Tools
                print("2. Testing /api/tools...")
                r_tools = await ac.get("/api/tools")
                assert r_tools.status_code == 200, f"Tools failed: {r_tools.text}"
                assert r_tools.json()["count"] == 7
                print("   PASS: /api/tools (7 tools discovered)")

                # 3. Chat: get_all_employees
                print("3. Testing /api/chat: get_all_employees...")
                r_chat1 = await ac.post("/api/chat", json={"prompt": "Show all employees"})
                assert r_chat1.status_code == 200, f"Chat failed: {r_chat1.text}"
                d1 = r_chat1.json()
                assert d1["status"] == "success"
                assert d1["selected_tools"] == ["get_all_employees"]
                assert d1["tool_executions"][0]["result"]["count"] >= 20
                print("   PASS: get_all_employees returned >= 20 records")

                # 4. Chat: get_employee_by_id
                print("4. Testing /api/chat: get_employee_by_id...")
                r_chat2 = await ac.post("/api/chat", json={"prompt": "Find employee ID 1001"})
                assert r_chat2.status_code == 200
                d2 = r_chat2.json()
                assert d2["status"] == "success"
                assert d2["selected_tools"] == ["get_employee_by_id"]
                assert d2["tool_executions"][0]["result"]["data"]["EmployeeID"] == 1001
                print("   PASS: get_employee_by_id 1001")

                # 5. Chat: search_employees
                print("5. Testing /api/chat: search_employees...")
                r_chat3 = await ac.post("/api/chat", json={"prompt": "Search employees in IT department"})
                assert r_chat3.status_code == 200
                d3 = r_chat3.json()
                assert d3["status"] == "success"
                assert d3["selected_tools"] == ["search_employees"]
                assert d3["tool_executions"][0]["result"]["count"] == 7
                print("   PASS: search_employees in IT (7 records)")

                # 6. Chat: get_department_list
                print("6. Testing /api/chat: get_department_list...")
                r_chat4 = await ac.post("/api/chat", json={"prompt": "Show all departments"})
                assert r_chat4.status_code == 200
                d4 = r_chat4.json()
                assert d4["status"] == "success"
                assert d4["selected_tools"] == ["get_department_list"]
                assert d4["tool_executions"][0]["result"]["count"] == 6
                print("   PASS: get_department_list (6 departments)")

                # 7. Chat: get_employee_count
                print("7. Testing /api/chat: get_employee_count...")
                r_chat5 = await ac.post("/api/chat", json={"prompt": "How many employees are there?"})
                assert r_chat5.status_code == 200
                d5 = r_chat5.json()
                assert d5["status"] == "success"
                assert d5["selected_tools"] == ["get_employee_count"]
                assert d5["tool_executions"][0]["result"]["count"] >= 20
                print("   PASS: get_employee_count")

                # 7b. Chat: get_employee_count with designation
                print("7b. Testing /api/chat: get_employee_count with designation...")
                r_chat5b = await ac.post("/api/chat", json={"prompt": "Count Senior Software Engineers"})
                assert r_chat5b.status_code == 200
                d5b = r_chat5b.json()
                assert d5b["status"] == "success"
                assert d5b["selected_tools"] == ["get_employee_count"]
                assert d5b["tool_executions"][0]["arguments"].get("designation") == "Senior Software Engineer"
                assert d5b["tool_executions"][0]["result"]["count"] == 1
                print("   PASS: get_employee_count with designation (Senior Software Engineer -> 1)")

                # 8. Chat: Sequential Multi-Tool
                print("8. Testing /api/chat: Multiple sequential tools...")
                r_chat6 = await ac.post(
                    "/api/chat",
                    json={"prompt": "Search employees in IT department and also count employees in IT"}
                )
                assert r_chat6.status_code == 200
                d6 = r_chat6.json()
                assert d6["status"] == "success"
                assert len(d6["tool_executions"]) == 2
                assert d6["selected_tools"] == ["search_employees", "get_employee_count"]
                print("   PASS: Sequential multi-tool execution")

                # 9. Chat: Missing Parameters
                print("9. Testing /api/chat: Missing parameters...")
                r_chat7 = await ac.post("/api/chat", json={"prompt": "Add a new employee named Robert Brown"})
                assert r_chat7.status_code == 200
                d7 = r_chat7.json()
                assert d7["status"] == "missing_parameters"
                assert "email" in d7["missing_parameters"]
                print("   PASS: Missing parameters detected correctly")

                # 10. Chat: Unsupported Prompt
                print("10. Testing /api/chat: Unsupported prompt...")
                r_chat8 = await ac.post("/api/chat", json={"prompt": "Tell me a joke about computers"})
                assert r_chat8.status_code == 200
                d8 = r_chat8.json()
                assert d8["status"] == "unsupported_prompt"
                print("   PASS: Unsupported prompt handled gracefully")

                # 11. Write Confirmation Flow
                print("11. Testing write confirmation flow...")
                write_prompt = "Add employee named Fast User with email fast.api.test@spillabs.com in IT as Software Engineer with salary 89000"
                chat_resp = await ac.post("/api/chat", json={"prompt": write_prompt})
                assert chat_resp.status_code == 200
                chat_data = chat_resp.json()
                assert chat_data["status"] == "confirmation_required"
                token = chat_data["pending_operation"]["confirmation_token"]
                assert token.startswith("CONF-")

                # Reject invalid token
                inv_resp = await ac.post("/api/confirm", json={"token": "CONF-000000"})
                assert inv_resp.status_code == 200
                assert inv_resp.json()["status"] == "validation_error"

                # Confirm valid token
                conf_resp = await ac.post("/api/confirm", json={"token": token})
                assert conf_resp.status_code == 200
                conf_data = conf_resp.json()
                assert conf_data["status"] == "success"
                created_id = conf_data["tool_executions"][0]["result"]["data"]["EmployeeID"]

                # Prevent reuse
                reuse_resp = await ac.post("/api/confirm", json={"token": token})
                assert reuse_resp.status_code == 200
                assert reuse_resp.json()["status"] == "validation_error"

                # Cleanup database
                subprocess.run(
                    ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", f"DELETE FROM Employees WHERE EmployeeID = {created_id};"],
                    check=True
                )
                print(f"   PASS: Write confirmation flow verified and cleaned up (EmployeeID {created_id})")

                # 12. Conversational Memory - Follow-up Pronoun Query
                print("12. Testing conversational memory follow-up query...")
                conv_id = "test-session-mem-1"
                r_mem1 = await ac.post("/api/chat", json={"prompt": "Find employee ID 1001", "conversation_id": conv_id})
                assert r_mem1.status_code == 200
                d_mem1 = r_mem1.json()
                assert d_mem1["status"] == "success"
                assert d_mem1["conversation_context"] is not None
                assert d_mem1["conversation_context"]["last_referenced_employee_id"] == 1001

                # Turn 2: "What is his phone number?"
                r_mem2 = await ac.post("/api/chat", json={"prompt": "What is his phone number?", "conversation_id": conv_id})
                assert r_mem2.status_code == 200
                d_mem2 = r_mem2.json()
                assert d_mem2["status"] == "success"
                assert "James Wilson" in d_mem2.get("resolved_reference", "")
                assert "077" in d_mem2.get("message", "") or "phone" in d_mem2.get("message", "").lower()
                print("   PASS: Conversational follow-up resolved reference and retrieved phone number")

                # 13. Conversational Memory - Session Reset
                print("13. Testing conversational memory reset...")
                r_del = await ac.delete(f"/api/conversations/{conv_id}")
                assert r_del.status_code == 200
                assert r_del.json()["status"] == "cleared"

                # Turn 3 after reset: "What is his phone number?" should fail gracefully with no context
                r_mem3 = await ac.post("/api/chat", json={"prompt": "What is his phone number?", "conversation_id": conv_id})
                assert r_mem3.status_code == 200
                d_mem3 = r_mem3.json()
                assert d_mem3["status"] in ["validation_error", "unsupported_prompt", "missing_parameters", "no_context"]
                assert "no previous employee" in d_mem3["message"] or "context" in d_mem3["message"].lower()
                print("   PASS: Context cleared successfully, subsequent pronoun handled gracefully")

        finally:
            await rust_manager.stop()

    asyncio.run(async_suite())

if __name__ == "__main__":
    test_api_full_suite()
    print("\nALL 11 BACKEND API TEST SCENARIOS PASSED CLEANLY!")
