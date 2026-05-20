import re

with open("x32_desk_restore/src/main.rs", "r") as f:
    text = f.read()

old_code = """    let commands: Vec<String> = io::BufReader::new(file.take(1024 * 1024))
        .lines()
        .map_while(std::result::Result::ok)
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();"""

new_code = """    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(X32Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too large",
        )));
    }

    let commands: Vec<String> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();"""

text = text.replace(old_code, new_code)

with open("x32_desk_restore/src/main.rs", "w") as f:
    f.write(text)
