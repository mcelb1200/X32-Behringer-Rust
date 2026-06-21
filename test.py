import subprocess
import shlex

cmd = "./target/debug/x32_usb --ip 127.0.0.1:10047 ls"
result = subprocess.run(shlex.split(cmd), capture_output=True, text=True)
print(f"STDOUT: '{result.stdout}'")
print(f"STDERR: '{result.stderr}'")
print(f"CODE: {result.returncode}")
