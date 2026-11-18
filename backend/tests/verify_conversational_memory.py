import asyncio
import sys
from httpx import AsyncClient, ASGITransport
from app.main import app
from app.services.rust_client import rust_manager

async def run_conversational_tests():
    print("=" * 70)
    print("EXECUTING 10 MULTI-TURN CONVERSATION REGRESSION SEQUENCES")
    print("=" * 70)

    await rust_manager.start()
    passed = 0
    total = 10

    try:
        async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as ac:
            # Sequence 1: Show employee ID 1001 -> What is his phone number?
            print("\n[Sequence 1] Single Employee ID -> What is his phone number?")
            c1 = "seq-1"
            r1_a = await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c1})
            assert r1_a.status_code == 200 and r1_a.json()["status"] == "success"
            r1_b = await ac.post("/api/chat", json={"prompt": "What is his phone number?", "conversation_id": c1})
            d1_b = r1_b.json()
            assert d1_b["status"] == "success", f"Failed: {d1_b}"
            assert "James Wilson" in d1_b.get("resolved_reference", "")
            assert "0771001006" in d1_b.get("message", "")
            print(f"  PASS: Resolved 'his' -> {d1_b['resolved_reference']} | Msg: {d1_b['message']}")
            passed += 1

            # Sequence 2: Show employee ID 1001 -> What is his email?
            print("\n[Sequence 2] Single Employee ID -> What is his email?")
            c2 = "seq-2"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c2})
            r2_b = await ac.post("/api/chat", json={"prompt": "What is his email?", "conversation_id": c2})
            d2_b = r2_b.json()
            assert d2_b["status"] == "success"
            assert "James Wilson" in d2_b.get("resolved_reference", "")
            assert "james.wilson@example.com" in d2_b.get("message", "")
            print(f"  PASS: Resolved 'his' -> {d2_b['resolved_reference']} | Msg: {d2_b['message']}")
            passed += 1

            # Sequence 3: Show employee ID 1001 -> Which department does he work in?
            print("\n[Sequence 3] Single Employee ID -> Which department does he work in?")
            c3 = "seq-3"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c3})
            r3_b = await ac.post("/api/chat", json={"prompt": "Which department does he work in?", "conversation_id": c3})
            d3_b = r3_b.json()
            assert d3_b["status"] == "success"
            assert "Engineering" in d3_b.get("message", "")
            print(f"  PASS: Resolved 'he' -> {d3_b['resolved_reference']} | Msg: {d3_b['message']}")
            passed += 1

            # Sequence 4: Show employee ID 1001 -> What is his designation?
            print("\n[Sequence 4] Single Employee ID -> What is his designation?")
            c4 = "seq-4"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c4})
            r4_b = await ac.post("/api/chat", json={"prompt": "What is his designation?", "conversation_id": c4})
            d4_b = r4_b.json()
            assert d4_b["status"] == "success"
            assert "Team Lead" in d4_b.get("message", "")
            print(f"  PASS: Resolved 'his' -> {d4_b['resolved_reference']} | Msg: {d4_b['message']}")
            passed += 1

            # Sequence 5: Find Senior Software Engineers -> What department does he work in?
            print("\n[Sequence 5] Unique Designation Search -> What department does he work in?")
            c5 = "seq-5"
            r5_a = await ac.post("/api/chat", json={"prompt": "Find Senior Software Engineers", "conversation_id": c5})
            assert r5_a.status_code == 200
            r5_b = await ac.post("/api/chat", json={"prompt": "What department does he work in?", "conversation_id": c5})
            d5_b = r5_b.json()
            assert d5_b["status"] == "success"
            assert "David Miller" in d5_b.get("resolved_reference", "")
            assert "IT" in d5_b.get("message", "")
            print(f"  PASS: Resolved 'he' -> {d5_b['resolved_reference']} | Msg: {d5_b['message']}")
            passed += 1

            # Sequence 6: Show Project Managers -> What are their phone numbers?
            print("\n[Sequence 6] Multiple Employees -> What are their phone numbers?")
            c6 = "seq-6"
            await ac.post("/api/chat", json={"prompt": "Show Project Managers", "conversation_id": c6})
            r6_b = await ac.post("/api/chat", json={"prompt": "What are their phone numbers?", "conversation_id": c6})
            d6_b = r6_b.json()
            assert d6_b["status"] == "success"
            assert "Jennifer Lee" in d6_b.get("message", "") and "William Thomas" in d6_b.get("message", "")
            print(f"  PASS: Resolved plural collection | Msg:\n{d6_b['message']}")
            passed += 1

            # Sequence 7: Show employee ID 1001 -> Update his phone number to 0771231234
            print("\n[Sequence 7] Conversational Update -> Update his phone number to 0771231234")
            c7 = "seq-7"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c7})
            r7_b = await ac.post("/api/chat", json={"prompt": "Update his phone number to 0771231234", "conversation_id": c7})
            d7_b = r7_b.json()
            assert d7_b["status"] == "confirmation_required", f"Expected confirmation_required: {d7_b}"
            assert d7_b["pending_operation"]["arguments"]["employee_id"] == 1001
            assert d7_b["pending_operation"]["arguments"]["phone"] == "0771231234"
            print(f"  PASS: Generated confirmation for Employee 1001 with phone: 0771231234")
            passed += 1

            # Sequence 8: Show employee ID 1001 -> What is his salary?
            print("\n[Sequence 8] Single Employee ID -> What is his salary?")
            c8 = "seq-8"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c8})
            r8_b = await ac.post("/api/chat", json={"prompt": "What is his salary?", "conversation_id": c8})
            d8_b = r8_b.json()
            assert d8_b["status"] == "success"
            assert "James Wilson" in d8_b.get("resolved_reference", "")
            assert "180000" in d8_b.get("message", "") or "180,000" in d8_b.get("message", "")
            print(f"  PASS: Resolved 'his' -> {d8_b['resolved_reference']} | Msg: {d8_b['message']}")
            passed += 1

            # Sequence 9: Show employee ID 1001 -> Start a new conversation -> What is his phone number?
            print("\n[Sequence 9] New Conversation Session Reset -> What is his phone number?")
            c9 = "seq-9"
            await ac.post("/api/chat", json={"prompt": "Show employee ID 1001", "conversation_id": c9})
            # Clear conversation context via DELETE
            r_del = await ac.delete(f"/api/conversations/{c9}")
            assert r_del.status_code == 200
            # Follow-up should fail gracefully with validation_error / no_context
            r9_b = await ac.post("/api/chat", json={"prompt": "What is his phone number?", "conversation_id": c9})
            d9_b = r9_b.json()
            assert d9_b["status"] in ["validation_error", "unsupported_prompt", "no_context"]
            assert "no previous employee" in d9_b.get("message", "").lower() or "context" in d9_b.get("message", "").lower()
            print(f"  PASS: Graceful failure after session reset | Status: {d9_b['status']} | Msg: {d9_b['message']}")
            passed += 1

            # Sequence 10: Show employees from Engineering -> What are their names?
            print("\n[Sequence 10] Plural Collection -> What are their names?")
            c10 = "seq-10"
            await ac.post("/api/chat", json={"prompt": "Show employees from Engineering", "conversation_id": c10})
            r10_b = await ac.post("/api/chat", json={"prompt": "What are their names?", "conversation_id": c10})
            d10_b = r10_b.json()
            assert d10_b["status"] == "success"
            assert "James Wilson" in d10_b.get("message", "")
            print(f"  PASS: Resolved plural names | Msg:\n{d10_b['message']}")
            passed += 1

    finally:
        await rust_manager.stop()

    print("\n" + "=" * 70)
    print(f"RESULT: ALL {passed}/{total} CONVERSATIONAL MEMORY SEQUENCES PASSED!")
    print("=" * 70)

if __name__ == "__main__":
    asyncio.run(run_conversational_tests())
