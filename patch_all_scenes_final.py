import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    if "x32_commander" in filepath:
        old_logic = """        let mut line = String::new();
        // Limit reading to 4096 bytes to prevent DoS via extremely long lines
        let len = reader.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            // Check if we hit the limit without reaching EOF on the underlying stream
            if file.limit() == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Configuration file exceeded the maximum length of 1MB during processing",
                ));
            }
            break;
        }

        let line = line.trim();"""

        new_logic = """        let mut byte_buf = Vec::new();
        match reader.by_ref().take(4096).read_until(b'\\n', &mut byte_buf) {
            Ok(0) => {
                if file.limit() == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Configuration file exceeded the maximum length of 1MB during processing",
                    ));
                }
                break;
            },
            Err(e) => return Err(e),
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\\n") {
                    let mut discard = Vec::with_capacity(1024);
                    loop {
                        discard.clear();
                        match reader.by_ref().take(1024).read_until(b'\\n', &mut discard) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                if discard.ends_with(b"\\n") {
                                    break;
                                }
                            }
                        }
                    }
                    eprintln!("Input line too long, discarded.");
                    continue;
                }
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }
        };

        let line = line_str.trim();"""

        new_content = content.replace(old_logic, new_logic)
        with open(filepath, 'w') as f:
            f.write(new_content)
        return True

    if "x32_tcp" in filepath:
        old_logic = """        let mut line = String::new();
        // Limit reading to 4096 bytes to prevent DoS via extremely long lines
        let len = reader.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            break; // Connection closed
        }

        if len == 4096 && !line.ends_with('\\n') {
            let error_msg = "Error: Input line too long (exceeds 4096 bytes). Connection closed.\\n";
            eprintln!("{}", error_msg.trim());
            stream.write_all(error_msg.as_bytes())?;
            break;
        }

        let trimmed_line = line.trim();"""

        new_logic = """        let mut byte_buf = Vec::new();
        match reader.by_ref().take(4096).read_until(b'\\n', &mut byte_buf) {
            Ok(0) => break, // Connection closed
            Err(_) => break, // Connection error
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\\n") {
                    let error_msg = "Error: Input line too long (exceeds 4096 bytes). Connection closed.\\n";
                    eprintln!("{}", error_msg.trim());
                    stream.write_all(error_msg.as_bytes())?;
                    break;
                }
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                let error_msg = "Error: Invalid UTF-8 sequence.\\n";
                stream.write_all(error_msg.as_bytes())?;
                continue;
            }
        };

        let trimmed_line = line_str.trim();"""

        new_content = content.replace(old_logic, new_logic)
        with open(filepath, 'w') as f:
            f.write(new_content)
        return True

    if "x32_replay" in filepath:
        old_logic = """        use std::io::{BufRead, Read};
        if stdin_lock.by_ref().take(4096).read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        let cmd = line.trim();"""

        new_logic = """        use std::io::{BufRead, Read};
        let mut byte_buf = Vec::new();
        match stdin_lock.by_ref().take(4096).read_until(b'\\n', &mut byte_buf) {
            Ok(0) | Err(_) => break,
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\\n") {
                    let mut discard = Vec::with_capacity(1024);
                    loop {
                        discard.clear();
                        match stdin_lock.by_ref().take(1024).read_until(b'\\n', &mut discard) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                if discard.ends_with(b"\\n") {
                                    break;
                                }
                            }
                        }
                    }
                    eprintln!("Input line too long, discarded.");
                    continue;
                }
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }
        };
        let cmd = line_str.trim();"""

        new_content = content.replace(old_logic, new_logic)
        # Also remove `let mut line = String::new();` and `line.clear();` just before this block
        new_content = new_content.replace("    let mut line = String::new();\n    loop {\n        line.clear();\n", "    loop {\n")
        with open(filepath, 'w') as f:
            f.write(new_content)
        return True

    # Common Logic for other files
    old_logic_pattern = r"""        let mut line = String::new\(\);\n        let len = (reader|stdin_lock)\.by_ref\(\)\.take\(4096\)\.read_line\(&mut line\)\?;\n        if len == 0 \{\n            break;\n        \}\n        let (line|trimmed_line) = line\.trim\(\);"""

    match = re.search(old_logic_pattern, content)
    if not match:
        old_logic_pattern2 = r"""        line\.clear\(\);\n        if reader\.by_ref\(\)\.take\(4096\)\.read_line\(&mut line\)\.is_err\(\) \|\| line\.is_empty\(\) \{\n            break;\n        \}\n        let trimmed_line = line\.trim\(\);"""
        match2 = re.search(old_logic_pattern2, content)
        if not match2:
            return False

        # x32_custom_layer case
        new_logic = f"""        let mut byte_buf = Vec::new();
        match reader.by_ref().take(4096).read_until(b'\\n', &mut byte_buf) {{
            Ok(0) => break, // EOF
            Err(e) => return Err(e.into()), // Propagate I/O errors properly
            Ok(len) => {{
                if len == 4096 && !byte_buf.ends_with(b"\\n") {{
                    // Line too long, discard remainder
                    let mut discard = Vec::with_capacity(1024);
                    loop {{
                        discard.clear();
                        let mut chunk_handle = reader.by_ref().take(1024);
                        match chunk_handle.read_until(b'\\n', &mut discard) {{
                            Ok(0) => break,
                            Err(e) => return Err(e.into()),
                            Ok(_) => {{
                                if discard.ends_with(b"\\n") {{
                                    break;
                                }}
                            }}
                        }}
                    }}
                    eprintln!("Input line too long, discarded.");
                    continue;
                }}
            }}
        }}

        let line_str = match std::str::from_utf8(&byte_buf) {{
            Ok(s) => s,
            Err(_) => {{
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }}
        }};

        let trimmed_line = line_str.trim();"""
        new_content = re.sub(old_logic_pattern2, new_logic, content)
        new_content = new_content.replace("    let mut line = String::new();\n    loop {\n", "    loop {\n")
        with open(filepath, 'w') as f:
            f.write(new_content)
        return True

    reader_name = match.group(1)
    var_name = match.group(2)

    new_logic = f"""        let mut byte_buf = Vec::new();
        let mut handle = {reader_name}.by_ref().take(4096);
        match handle.read_until(b'\\n', &mut byte_buf) {{
            Ok(0) => break, // EOF
            Err(e) => return Err(e.into()), // Propagate I/O errors properly
            Ok(len) => {{
                if len == 4096 && !byte_buf.ends_with(b"\\n") {{
                    // Line too long, discard remainder
                    let mut discard = Vec::with_capacity(1024);
                    loop {{
                        discard.clear();
                        let mut chunk_handle = {reader_name}.by_ref().take(1024);
                        match chunk_handle.read_until(b'\\n', &mut discard) {{
                            Ok(0) => break,
                            Err(e) => return Err(e.into()),
                            Ok(_) => {{
                                if discard.ends_with(b"\\n") {{
                                    break;
                                }}
                            }}
                        }}
                    }}
                    eprintln!("Input line too long, discarded.");
                    continue;
                }}
            }}
        }}

        let line_str = match std::str::from_utf8(&byte_buf) {{
            Ok(s) => s,
            Err(_) => {{
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }}
        }};

        let {var_name} = line_str.trim();"""

    new_content = re.sub(old_logic_pattern, new_logic, content)
    with open(filepath, 'w') as f:
        f.write(new_content)
    return True

def process_command(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Match the existing logic in x32_command and xair_command
    old_logic_pattern = r"""            let mut handle = stdin_lock\.by_ref\(\)\.take\(4096\);\n            if handle\.read_line\(&mut line_buf\)\.is_err\(\) \|\| line_buf\.is_empty\(\) \{\n                break;\n            \}\n            if !line_buf\.ends_with\('\\n'\) && line_buf\.len\(\) == 4096 \{\n                // If it doesn't end with a newline and hit the length limit, the line was too long\.\n                // Clear the rest of the line from stdin to avoid processing partial commands\.\n                let mut discard = Vec::with_capacity\(1024\);\n                loop \{\n                    discard\.clear\(\);\n                    let mut chunk_handle = stdin_lock\.by_ref\(\)\.take\(1024\);\n                    match chunk_handle\.read_until\(b'\\n', &mut discard\) \{\n                        Ok\(0\) \| Err\(_\) => break,\n                        Ok\(_\) => \{\n                            if discard\.ends_with\(b"\\n"\) \{\n                                break;\n                            \}\n                        \}\n                    \}\n                \}\n                eprintln!\("Input line too long, discarded\."\);\n                continue;\n            \}\n            let line = line_buf\.trim\(\);"""

    new_logic = """            let mut byte_buf = Vec::new();
            let mut handle = stdin_lock.by_ref().take(4096);
            match handle.read_until(b'\\n', &mut byte_buf) {
                Ok(0) => break, // EOF
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                },
                Ok(len) => {
                    if len == 4096 && !byte_buf.ends_with(b"\\n") {
                        let mut discard = Vec::with_capacity(1024);
                        loop {
                            discard.clear();
                            let mut chunk_handle = stdin_lock.by_ref().take(1024);
                            match chunk_handle.read_until(b'\\n', &mut discard) {
                                Ok(0) | Err(_) => break,
                                Ok(_) => {
                                    if discard.ends_with(b"\\n") {
                                        break;
                                    }
                                }
                            }
                        }
                        eprintln!("Input line too long, discarded.");
                        continue;
                    }
                }
            }

            let line_str = match std::str::from_utf8(&byte_buf) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Invalid UTF-8 sequence in input, discarded.");
                    continue;
                }
            };

            let line = line_str.trim();"""

    if re.search(old_logic_pattern, content):
        new_content = re.sub(old_logic_pattern, new_logic, content)

        # Remove the `let mut line_buf = String::new();` right above it
        new_content = new_content.replace("            let mut line_buf = String::new();\n            let mut byte_buf = Vec::new();", "            let mut byte_buf = Vec::new();")

        with open(filepath, 'w') as f:
            f.write(new_content)
        return True
    return False

files = [
    'xair_set_scene/src/main.rs',
    'xair_get_scene/src/main.rs',
    'x32_set_scene/src/main.rs',
    'x32_get_scene/src/main.rs',
    'x32_custom_layer/src/main.rs',
    'x32_commander/src/main.rs',
    'x32_tcp/src/main.rs',
    'x32_replay/src/main.rs',
]

for file in files:
    if process_file(file):
        print(f"Patched {file}")
    else:
        print(f"Failed to patch {file}")

files = ['x32_command/src/main.rs', 'xair_command/src/main.rs']
for file in files:
    if process_command(file):
        print(f"Patched {file}")
    else:
        print(f"Failed to patch {file}")
