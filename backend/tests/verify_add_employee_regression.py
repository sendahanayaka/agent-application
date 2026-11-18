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
    print("RUNNING REQUIRED REGRESSION TESTS")
    print("=" * 80)

    # 1. Primary failing prompt from screenshot
    p1 = "add new employee named Test User in Engineering as Software Engineer with email testuser@example.com and salary 95000"
    print(f"\n[Test 1] {p1}")
    r1 = chat(p1)
    print("Status:", r1["status"])
    print("Intents:", r1["detected_intents"])
    print("Tools:", r1["selected_tools"])
    assert r1["status"] == "confirmation_required", f"Expected confirmation_required, got {r1['status']}"
    assert r1["selected_tools"] == ["add_employee"], f"Expected only ['add_employee'], got {r1['selected_tools']}"
    assert r1["detected_intents"] == ["add_employee"], f"Expected only ['add_employee'], got {r1['detected_intents']}"
    
    pending = r1["pending_operation"]
    token = pending["confirmation_token"]
    args = pending["arguments"]
    print("Token:", token)
    print("Args:", args)
    assert args["first_name"] == "Test"
    assert args["last_name"] == "User"
    assert args["email"] == "testuser@example.com"
    assert args["department_id"] == 5  # Engineering
    assert args["designation_id"] == 1 # Software Engineer
    assert args["salary"] == 95000.0
    print("PASS: Prompt 1 parsed all 5 parameters correctly and isolated add_employee!")

    # Verify no database insertion happened yet
    sql_check = subprocess.run(
        ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", "SET NOCOUNT ON; SELECT COUNT(1) FROM Employees WHERE Email = 'testuser@example.com';"],
        capture_output=True, text=True, check=True
    )
    assert "0" in sql_check.stdout, "Employee should NOT be inserted before confirmation!"
    print("PASS: No database insertion occurred prior to confirmation.")

    # Confirm operation
    c1 = confirm(token)
    assert c1["status"] == "success", f"Confirmation failed: {c1}"
    created_id = c1["tool_executions"][0]["result"]["data"]["EmployeeID"]
    print(f"PASS: Confirmed operation successfully! Created EmployeeID: {created_id}")

    # Verify in SQL Server
    sql_verify = subprocess.run(
        ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", f"SET NOCOUNT ON; SELECT FirstName, LastName, Email, Salary, DepartmentID, DesignationID FROM Employees WHERE EmployeeID = {created_id};"],
        capture_output=True, text=True, check=True
    )
    print("SQL Server record:\n", sql_verify.stdout.strip())
    assert "testuser@example.com" in sql_verify.stdout

    # Clean up test record
    subprocess.run(
        ["sqlcmd", "-S", "localhost", "-E", "-d", "InternAgenticDB", "-Q", f"DELETE FROM Employees WHERE EmployeeID = {created_id};"],
        check=True
    )
    print("PASS: Cleaned up test record from database.")

    # 2. Add employee named Test User with salary 95000 (missing email, dept, desig)
    p2 = "Add employee named Test User with salary 95000"
    print(f"\n[Test 2] {p2}")
    r2 = chat(p2)
    print("Status:", r2["status"], "| Missing:", r2.get("missing_parameters"))
    assert r2["status"] == "missing_parameters"
    assert "salary" not in r2["missing_parameters"] # salary was captured!
    assert "email" in r2["missing_parameters"]

    # 3. Add employee with salary: 95000
    p3 = "Add employee with salary: 95000"
    print(f"\n[Test 3] {p3}")
    r3 = chat(p3)
    print("Status:", r3["status"], "| Missing:", r3.get("missing_parameters"))
    assert r3["status"] == "missing_parameters"
    assert "salary" not in r3["missing_parameters"]

    # 4. Add employee with salary of 95000
    p4 = "Add employee with salary of 95000"
    print(f"\n[Test 4] {p4}")
    r4 = chat(p4)
    print("Status:", r4["status"], "| Missing:", r4.get("missing_parameters"))
    assert r4["status"] == "missing_parameters"
    assert "salary" not in r4["missing_parameters"]

    # 5. Add employee with pay 95000
    p5 = "Add employee with pay 95000"
    print(f"\n[Test 5] {p5}")
    r5 = chat(p5)
    print("Status:", r5["status"], "| Missing:", r5.get("missing_parameters"))
    assert r5["status"] == "missing_parameters"
    assert "salary" not in r5["missing_parameters"]

    # 6. Add employee with salary $95,000
    p6 = "Add employee with salary $95,000"
    print(f"\n[Test 6] {p6}")
    r6 = chat(p6)
    print("Status:", r6["status"], "| Missing:", r6.get("missing_parameters"))
    assert r6["status"] == "missing_parameters"
    assert "salary" not in r6["missing_parameters"]

    # 7. Add employee without salary — correctly report missing salary
    p7 = "Add employee named Test User with email test@example.com in IT as Software Engineer"
    print(f"\n[Test 7] {p7}")
    r7 = chat(p7)
    print("Status:", r7["status"], "| Missing:", r7.get("missing_parameters"))
    assert r7["status"] == "missing_parameters"
    assert "salary" in r7["missing_parameters"]
    print("PASS: Correctly reported missing salary!")

    # 8. Add employee with invalid or negative salary — reject it
    p8 = "Add employee named Test User with email test@example.com in IT as Software Engineer with salary -5000"
    print(f"\n[Test 8] {p8}")
    r8 = chat(p8)
    print("Status:", r8["status"], "| Missing:", r8.get("missing_parameters"))
    assert r8["status"] == "missing_parameters"
    assert "salary" in r8["missing_parameters"]
    print("PASS: Correctly rejected negative salary as missing/invalid!")

    # 9. Update employee ID 1001 salary to 95000
    p9 = "Update employee ID 1001 salary to 95000"
    print(f"\n[Test 9] {p9}")
    r9 = chat(p9)
    print("Status:", r9["status"], "| Tools:", r9.get("selected_tools"))
    assert r9["status"] == "confirmation_required"
    assert r9["selected_tools"] == ["update_employee"]
    print("Token:", r9["pending_operation"]["confirmation_token"])
    print("PASS: Update employee generated confirmation token for update_employee without search_employees!")

    # 10. Search Software Engineers — must still select search_employees
    p10 = "Search Software Engineers"
    print(f"\n[Test 10] {p10}")
    r10 = chat(p10)
    print("Status:", r10["status"], "| Tools:", r10.get("selected_tools"))
    assert r10["status"] == "success"
    assert r10["selected_tools"] == ["search_employees"]
    assert r10["tool_executions"][0]["arguments"]["designation"] == "Software Engineer"
    print("PASS: Search Software Engineers routes cleanly to search_employees!")

    print("\nALL 10 REGRESSION SCENARIOS COMPLETED SUCCESSFULLY!")

if __name__ == "__main__":
    run_tests()
