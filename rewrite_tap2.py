import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("let std_socket = create_socket(&args.ip, 500).context(\"Failed to create socket\")?;\n    std_socket.set_nonblocking(true)?;\n    let socket = tokio::net::UdpSocket::from_std(std_socket)?;\n\n    handle_client(Arc::new(socket), args).await", """let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    handle_client(client, args).await""")

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
