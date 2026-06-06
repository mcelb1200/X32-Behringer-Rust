import re

with open("libs/x32_lib/src/client.rs", "r") as f:
    text = f.read()

query_node = """
    /// Queries the `/node` command for a given path and returns the response string.
    pub async fn query_node(&self, node_path: &str) -> Result<String> {
        let mut rx = self.msg_tx.subscribe();
        self.send_message("/node", vec![OscArg::String(node_path.to_string())]).await?;

        let timeout_dur = Duration::from_secs(2);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_dur {
            match time::timeout(timeout_dur - start.elapsed(), rx.recv()).await {
                Ok(Ok(msg)) => {
                    if msg.path == "/node" || msg.path == "node" {
                        if let Some(OscArg::String(response_str)) = msg.args.first() {
                            return Ok(response_str.clone());
                        }
                    }
                }
                _ => continue,
            }
        }
        Err(OscError::ParseError("Query /node timeout".to_string()).into())
    }
"""

text = text.replace("impl MixerClient {\n", "impl MixerClient {\n" + query_node)

with open("libs/x32_lib/src/client.rs", "w") as f:
    f.write(text)
