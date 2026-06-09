import sys

def add_allow(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    if '#![allow(deprecated)]' not in content:
        content = '#![allow(deprecated)]\n' + content
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"Fixed {filepath}")

add_allow('libs/x32_lib/src/tests.rs')
add_allow('libs/x32_lib/tests/socket.rs')
add_allow('libs/x32_lib/tests/fx_tests.rs')
