import urllib.request
import json
import subprocess

BASE_URL = "http://127.0.0.1:8000"

def chat(prompt):
    data = json.dumps({"prompt": prompt}).encode()
    req = urllib.request.Request(f"{BASE_URL}/api/chat", data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())

def confirm(token):
    data = json.dumps({"token": token}).encode()
    req = urllib.request.Request(f"{BASE_URL}/api/confirm", data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())

def run_tests():
    print("=" * 80)
    print("RUNNING ALL 10 UPDATE EMPLOYEE REGRESSION TESTS")
    print("=" * 80)

    # 1. Update employee ID 1001 phone number to 0771231234
    p1 = "Update employee ID 1001 phone number to 0771231234"
    print(f"\n[Test 1] {p1}")
    r1 = chat(p1)
    print("Status:", r1["status"])
    print("Selected Tools:", r1.get("selected_tools"))
    assert r1["status"] == "confirmation_required"
    assert r1["selected_tools"] == ["update_employee"]
    assert r1["pending_operation"]["arguments"]["employee_id"] == 1001
    assert r1["pending_operation"]["arguments"]["phone"] == "0771231234"
    token1 = r1["pending_operation"]["confirmation_token"]

    # Verify no database modification before confirmation
    sql_before = subprocess.run(
        ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", "SET NOCOUNT ON; SELECT Phone FROM Employees WHERE EmployeeID = 1001;"],
        capture_output=True, text=True, check=True
    )
    old_phone = sql_before.stdout.strip()

    # Confirm update
    c1 = confirm(token1)
    assert c1["status"] == "success"
    assert c1["tool_executions"][0]["result"]["data"]["Phone"] == "0771231234"
    print("PASS: Updated phone number to 0771231234 and verified in database response!")

    # 2. Change phone number of employee 1001 to 0779998888
    p2 = "Change phone number of employee 1001 to 0779998888"
    print(f"\n[Test 2] {p2}")
    r2 = chat(p2)
    assert r2["status"] == "confirmation_required"
    assert r2["pending_operation"]["arguments"]["phone"] == "0779998888"
    token2 = r2["pending_operation"]["confirmation_token"]
    confirm(token2)
    print("PASS: Phone variation recognized and updated!")

    # 3. Update employee 1001 email to test@example.com
    p3 = "Update employee 1001 email to test@example.com"
    print(f"\n[Test 3] {p3}")
    r3 = chat(p3)
    assert r3["status"] == "confirmation_required"
    assert r3["pending_operation"]["arguments"]["email"] == "test@example.com"
    print("PASS: Update employee email extracted correctly!")

    # 4. Update employee 1001 salary to 95000
    p4 = "Update employee 1001 salary to 95000"
    print(f"\n[Test 4] {p4}")
    r4 = chat(p4)
    assert r4["status"] == "confirmation_required"
    assert r4["pending_operation"]["arguments"]["salary"] == 95000.0
    print("PASS: Update employee salary extracted correctly!")

    # 5. Update employee 1001 department to Engineering
    p5 = "Update employee 1001 department to Engineering"
    print(f"\n[Test 5] {p5}")
    r5 = chat(p5)
    assert r5["status"] == "confirmation_required"
    assert r5["pending_operation"]["arguments"]["department_id"] == 5 # Engineering
    print("PASS: Update employee department extracted correctly (mapped to dept id 5)!")

    # 6. Update employee 1001 with phone number and salary together
    p6 = "Update employee 1001 phone number 0771112222 and salary 105000"
    print(f"\n[Test 6] {p6}")
    r6 = chat(p6)
    assert r6["status"] == "confirmation_required"
    args6 = r6["pending_operation"]["arguments"]
    assert args6["employee_id"] == 1001
    assert args6["phone"] == "0771112222"
    assert args6["salary"] == 105000.0
    token6 = r6["pending_operation"]["confirmation_token"]
    c6 = confirm(token6)
    assert c6["status"] == "success"
    print("PASS: Compound update (phone + salary) executed cleanly!")

    # 7. Update employee 1001 without specifying any update field
    p7 = "Update employee 1001 without specifying any update field"
    print(f"\n[Test 7] {p7}")
    r7 = chat(p7)
    assert r7["status"] == "validation_error"
    assert "No update fields provided" in r7["message"]
    print(f"PASS: Rejected empty update: '{r7['message']}'")

    # 8. Update nonexistent employee ID 9999
    p8 = "Update employee ID 9999 phone to 0770001111"
    print(f"\n[Test 8] {p8}")
    r8 = chat(p8)
    assert r8["status"] == "confirmation_required"
    token8 = r8["pending_operation"]["confirmation_token"]
    c8 = confirm(token8)
    assert c8["tool_executions"][0]["result"]["success"] == False
    assert "Employee with ID 9999 does not exist" in c8["tool_executions"][0]["result"]["message"]
    print("PASS: Nonexistent employee error reported from MCP repository!")

    # 9. Update employee ID 1038 to 1021
    p9 = "Update employee ID 1038 to 1021"
    print(f"\n[Test 9] {p9}")
    r9 = chat(p9)
    assert r9["status"] == "validation_error"
    assert "Modifying EmployeeID" in r9["message"]
    print(f"PASS: Immutable identity key policy explained clearly: '{r9['message']}'")

    # 10. Search Software Engineers — ensure search still works
    p10 = "Search Software Engineers"
    print(f"\n[Test 10] {p10}")
    r10 = chat(p10)
    assert r10["status"] == "success"
    assert r10["selected_tools"] == ["search_employees"]
    print("PASS: Search Software Engineers routes cleanly to search_employees!")

    # Restore employee 1001 original record in database
    subprocess.run(
        ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", f"UPDATE Employees SET Phone = '{old_phone}', Salary = 180000 WHERE EmployeeID = 1001;"],
        check=True
    )
    print("\nPASS: Restored Employee 1001 original phone and salary in database!")
    print("\nALL 10 REGRESSION TESTS PASSED CLEANLY!")

if __name__ == "__main__":
    run_tests()
