import sys

with open('tools/x32_usb/tests/integration_test.rs', 'r') as f:
    content = f.read()

# Make the thread handles ignore send errors if the server exited early.
content = content.replace("tx.send(()).unwrap();", "let _ = tx.send(());")

with open('tools/x32_usb/tests/integration_test.rs', 'w') as f:
    f.write(content)
print("Fixed usb test")
