with open("tools/x32_desk_save/src/main.rs", "r") as f:
    content = f.read()

content = content.replace("use std::io::{BufRead, BufWriter, Write};", "use std::io::{BufRead, BufWriter, Read, Write};")

with open("tools/x32_desk_save/src/main.rs", "w") as f:
    f.write(content)
