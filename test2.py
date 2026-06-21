import subprocess
import shlex

cmd = "cargo test -p x32_usb --test integration_test test_not_connected"
result = subprocess.run(shlex.split(cmd), capture_output=True, text=True)
print(f"STDOUT: '{result.stdout}'")
print(f"STDERR: '{result.stderr}'")
