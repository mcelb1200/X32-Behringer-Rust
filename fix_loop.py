with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

import re

# Find the loop block in auto mode
loop_start = text.find("        loop {\n            // Send keepalives every 9 seconds")
loop_end = text.find("    } else {\n        println!(\"X32Tap - Manual Mode\");")

if loop_start != -1 and loop_end != -1:
    old_loop = text[loop_start:loop_end]
    new_loop = r"""        loop {
            // Send keepalives every 9 seconds
            let now = Instant::now();
            if now.duration_since(last_keepalive).as_secs() >= 9 {
                // Keep the connection alive
                let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
                if let Err(e) = client.send_message(&xremote_msg.path, xremote_msg.args.clone()).await {
                    eprintln!("Failed to send /xremote: {}", e);
                }

                let meter_req = OscMessage::new(
                    "/meters".to_string(),
                    vec![
                        OscArg::String("/meters/6".to_string()),
                        OscArg::Int(0),
                        OscArg::Int(0),
                        OscArg::Int((args.channel - 1) as i32),
                    ],
                );
                if let Err(e) = client.send_message(&meter_req.path, meter_req.args.clone()).await {
                    eprintln!("Failed to send /meters request: {}", e);
                }

                last_keepalive = now;
            }

            // Read UDP packets
            if let Ok(Ok(msg)) = timeout(Duration::from_millis(100), rx.recv()).await {
                if msg.path == "/meters/6" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        if data.len() >= 16 {
                            let mut f_bytes = [0u8; 4];
                            if let Some(slice) = data.get(12..16) {
                                f_bytes.copy_from_slice(slice);
                            } else {
                                continue;
                            }
                            let level = f32::from_le_bytes(f_bytes);

                            if level > args.threshold {
                                if !was_above_threshold {
                                    let tap_time = Instant::now();
                                    if let Some(last) = last_tap {
                                        let delta = tap_time.duration_since(last);
                                        let delta_ms = delta.as_millis() as f32;

                                        if delta_ms > 60.0 {
                                            let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                            let tempo_ms = (f_val * 3000.0) as i32;
                                            println!("Auto Tap: {}ms (level: {:.2})", tempo_ms, level);

                                            let update_msg = OscMessage::new(
                                                address.clone(),
                                                vec![OscArg::Float(f_val)],
                                            );
                                            if let Err(e) = client.send_message(&update_msg.path, update_msg.args.clone()).await {
                                                eprintln!("Failed to update FX parameter: {}", e);
                                            }
                                            last_tap = Some(tap_time);
                                        }
                                    } else {
                                        println!("First auto tap... (level: {:.2})", level);
                                        last_tap = Some(tap_time);
                                    }
                                    was_above_threshold = true;
                                }
                            } else {
                                was_above_threshold = false;
                            }
                        }
                    }
                }
            }
        }
"""
    text = text.replace(old_loop, new_loop)
    with open("tools/x32_tap/src/main.rs", "w") as f:
        f.write(text)
