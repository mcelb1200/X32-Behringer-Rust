import re

def rewrite_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # In x32_lib/src/scene_parse.rs:
    # let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    # To array avoiding allocation:
    content = content.replace(
"""    let mut messages = Vec::new();
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() {
        return messages;
    }

    match parts.as_slice() {""",
"""    let mut messages = Vec::new();
    let mut parts = [""; 8];
    let mut num_parts = 0;
    for part in path.trim_start_matches('/').split('/') {
        if num_parts < 8 {
            parts[num_parts] = part;
            num_parts += 1;
        } else {
            break;
        }
    }
    if num_parts == 0 {
        return messages;
    }

    match &parts[..num_parts] {"""
    )
    with open(filepath, 'w') as f:
        f.write(content)

rewrite_file('x32_lib/src/scene_parse.rs')
