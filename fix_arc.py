with open("tools/x32_jog4xlive/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace("let client = std::sync::Arc::new(MixerClient::connect(&addr, true).await?);", "let client = MixerClient::connect(&addr, true).await?;")

with open("tools/x32_jog4xlive/src/lib.rs", "w") as f:
    f.write(content)
