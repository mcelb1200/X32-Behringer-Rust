import re

def rewrite_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # In x32_set_preset/src/main.rs:
    # let parts: Vec<&str> = addr.split('/').collect();
    content = content.replace(
"""        if prefix.starts_with("/ch/") {
            let ch_str = &prefix[4..6]; // "01"
            let parts: Vec<&str> = addr.split('/').collect();
            if parts.len() >= 4 {
                // "", "headamp", "000", "gain"
                return format!("/headamp/{}/{}", ch_str, parts[3]);
            }
        }""",
"""        if prefix.starts_with("/ch/") {
            let ch_str = &prefix[4..6]; // "01"
            let mut parts = addr.split('/');
            let _ = parts.next(); // ""
            let _ = parts.next(); // "headamp"
            let _ = parts.next(); // "000"
            if let Some(part3) = parts.next() {
                // "", "headamp", "000", "gain"
                return format!("/headamp/{}/{}", ch_str, part3);
            }
        }"""
    )
    with open(filepath, 'w') as f:
        f.write(content)

rewrite_file('x32_set_preset/src/main.rs')
