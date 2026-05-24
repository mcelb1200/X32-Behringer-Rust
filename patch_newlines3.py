import glob
for file in glob.glob('*/src/main.rs'):
    with open(file, 'r') as f:
        content = f.read()
    new_content = content.replace("match handle.read_until(b'\n', &mut byte_buf)", "match handle.read_until(b'\\n', &mut byte_buf)")
    new_content = new_content.replace("match chunk_handle.read_until(b'\n', &mut discard)", "match chunk_handle.read_until(b'\\n', &mut discard)")
    with open(file, 'w') as f:
        f.write(new_content)
