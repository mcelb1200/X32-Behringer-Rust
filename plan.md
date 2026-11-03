1. **Add `send` method to `MixerClient`**
   - Add `pub async fn send(&self, msg: OscMessage) -> Result<()>` to `libs/x32_lib/src/client.rs`. This just calls `self.transport.send(msg).await`.

2. **Update `apps/x32_reaper/src/lib.rs` and other usages**
   - Replace occurrences of `x_client.send_message(&msg.path, msg.args.clone()).await` with `x_client.send(msg).await`.
   - Wait, if `msg` is reused, maybe they can't consume it?
   - Let's check `apps/x32_reaper/src/lib.rs:161-166`:
     ```rust
     let msg = OscMessage {
         path: format!("/config/userctrl/C/enc/{}", i),
         args: vec![OscArg::String(mp[i - 1].to_string())],
     };
     let _ = x_client.send_message(&msg.path, msg.args.clone()).await;
     ```
     Here `msg` is created and used only once! It doesn't need to be cloned at all.
   - Wait, what about line 288?
     ```rust
        path_buf.clear();
        write!(&mut path_buf, "/ch/{:02}/mix/fader", i).expect("Failed to format OSC path");
        let msg = OscMessage {
            path: path_buf.clone(),
            args: vec![OscArg::Float(track.fader)],
        };
        let _ = x_client.send_message(&msg.path, msg.args.clone()).await;
     ```
     Same here! `msg` is dropped immediately after. So `x_client.send(msg).await` is perfectly fine!
   - This eliminates multiple string clones and vector clones inside hot loops.

3. **Complete Pre-Commit Steps**
   - Use `pre_commit_instructions` and follow them to format, check, and test the project.

4. **Submit**
   - Submit the PR with appropriate `Bolt` formatting.
