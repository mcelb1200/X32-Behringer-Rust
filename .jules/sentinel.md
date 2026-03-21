## 2024-05-24 - [DoS via Unbounded Line Reads]
**Vulnerability:** The `x32_tcp` application used `BufRead::read_line` to accept incoming commands without a size limit. An attacker could establish a connection and send a continuous stream of bytes without a newline character, causing the application to continuously allocate memory.
**Learning:** Rust's `BufRead::read_line` will read until it encounters a newline or EOF. If an input source is untrusted, this can lead to unbounded memory allocation and eventual Out Of Memory (OOM) crashes.
**Prevention:** When reading lines from untrusted sources (e.g., network connections), always limit the maximum number of bytes read. This can be achieved efficiently using the `take()` adapter on the reader (e.g., `reader.by_ref().take(LIMIT).read_line(&mut buf)`).

## 2024-03-21 - [DoS via Unbounded read_line/lines() from STDIN]
**Vulnerability:** Several CLI tools (`x32_get_scene`, `x32_set_scene`) used unbounded `io::stdin().read_line()` or `io::stdin().lock().lines()` to read piped inputs or manual inputs. This can lead to unbounded memory allocation and Out-Of-Memory (OOM) crashes if a script or an attacker pipes a massive stream of data without a newline.
**Learning:** The same vulnerability pattern identified in networked tools (like `x32_tcp`) also applies to local CLI tools processing piped input. An unbounded `read_line` or `.lines()` iterator continuously allocates memory until it finds a `\n` or EOF.
**Prevention:** When reading lines from any untrusted or uncontrolled source (including STDIN, which can be piped), limit the maximum number of bytes read using the `take()` adapter on the reader (e.g. `stdin_lock.by_ref().take(4096).read_line(&mut buf)`).
