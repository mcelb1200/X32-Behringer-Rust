import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

main_old = r"""    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    std_socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(std_socket)?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    let info_bytes = info_msg.to_bytes().context("Failed to serialize /info")?;
    socket.send(&info_bytes).await.context("Failed to send /info")?;

    let mut buf = [0; 1024];
    match tokio::time::timeout(std::time::Duration::from_secs(2), socket.recv(&mut buf)).await {
        Ok(Ok(_)) => println!("Connected."),
        _ => eprintln!("Warning: Did not receive response to /info. The IP address might be incorrect or the console is unreachable."),
    }

    let socket = Arc::new(socket);

    handle_client(socket, args).await"""

main_new = r"""    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    let mut rx = client.subscribe();
    client.send_message("/info", vec![]).await.context("Failed to send /info")?;

    match tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv()).await {
        Ok(Ok(_)) => println!("Connected."),
        _ => eprintln!("Warning: Did not receive response to /info. The IP address might be incorrect or the console is unreachable."),
    }

    handle_client(client, args).await"""

text = text.replace(main_old, main_new)

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
