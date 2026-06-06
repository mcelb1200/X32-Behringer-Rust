import re

with open("tools/x32_set_preset/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("let socket = create_socket(&args.ip, 500)?;", """let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);""")

text = text.replace("client.send_message(&msg.path, msg.args.clone()).await?;", "client.send_message(&msg.path, msg.args.clone()).await?;")
# Oh I replaced `socket.send` but `client` was not defined.
text = text.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args.clone()).await?;")

with open("tools/x32_set_preset/src/main.rs", "w") as f:
    f.write(text)
