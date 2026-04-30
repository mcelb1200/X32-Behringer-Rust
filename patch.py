with open("x32_lib/src/scene_parse.rs", "r") as f:
    c = f.read()

c = c.replace('else if arg_str == "-oo" || arg_str.contains(\'.\') {', 'else if arg_str == "-oo" || arg_str.contains(\'.\') || arg_str.contains(\'k\') || arg_str.contains(\'K\') {')

with open("x32_lib/src/scene_parse.rs", "w") as f:
    f.write(c)
