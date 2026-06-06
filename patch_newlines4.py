import glob
for file in glob.glob('*/src/main.rs'):
    with open(file, 'r') as f:
        content = f.read()
    new_content = content.replace("match reader.by_ref().take(4096).read_until(b'\n', &mut byte_buf)", "match reader.by_ref().take(4096).read_until(b'\\n', &mut byte_buf)")
    new_content = new_content.replace("match reader.by_ref().take(1024).read_until(b'\n', &mut discard)", "match reader.by_ref().take(1024).read_until(b'\\n', &mut discard)")
    new_content = new_content.replace("match stdin_lock.by_ref().take(4096).read_until(b'\n', &mut byte_buf)", "match stdin_lock.by_ref().take(4096).read_until(b'\\n', &mut byte_buf)")
    new_content = new_content.replace("match stdin_lock.by_ref().take(1024).read_until(b'\n', &mut discard)", "match stdin_lock.by_ref().take(1024).read_until(b'\\n', &mut discard)")
    with open(file, 'w') as f:
        f.write(new_content)
