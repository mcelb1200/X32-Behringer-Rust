with open("tools/x32_udp/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace("timeout_duration - start.elapsed()", "timeout_duration.saturating_sub(start.elapsed())")

with open("tools/x32_udp/src/lib.rs", "w") as f:
    f.write(content)
