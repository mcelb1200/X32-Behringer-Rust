with open("tools/x32_desk_save/src/main.rs", "r") as f:
    text = f.read()

old_block = r"""    } else if let Some(pattern_file) = &args.pattern_file {
        let file = File::open(pattern_file)?;
        let reader = std::io::BufReader::new(file);
        reader.lines().filter_map(|l| l.ok()).collect()
    } else {"""

new_block = r"""    } else if let Some(pattern_file) = &args.pattern_file {
        let file = File::open(pattern_file)?;
        let reader = std::io::BufReader::new(file);
        reader.lines()
            .filter_map(|l| l.ok())
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect()
    } else {"""

text = text.replace(old_block, new_block)

with open("tools/x32_desk_save/src/main.rs", "w") as f:
    f.write(text)
