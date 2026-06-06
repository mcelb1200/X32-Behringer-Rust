import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

main_re = re.compile(r'    let std_socket = create_socket\(&args\.ip, 500\).*?handle_client\(.*?\.await', re.DOTALL)
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

text = main_re.sub(main_new, text)

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
