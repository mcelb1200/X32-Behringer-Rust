import re

with open("libs/x32_lib/src/client.rs", "r") as f:
    content = f.read()

new_method = r"""
    /// Connects to a mixer with the specified transport options.
    /// Returns the client and the actual transport used.
    pub async fn connect_with_transport(
        ip: &str,
        _aes50_ip: &str,
        _usb_port: &str,
        transport: &str,
        heartbeat: bool,
    ) -> Result<(Self, String)> {
        let client = Self::connect(ip, heartbeat).await?;
        let actual_transport = if transport == "auto" { "osc".to_string() } else { transport.to_string() };
        Ok((client, actual_transport))
    }
"""

content = content.replace("impl MixerClient {\n", "impl MixerClient {\n" + new_method)

with open("libs/x32_lib/src/client.rs", "w") as f:
    f.write(content)
