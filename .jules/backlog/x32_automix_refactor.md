# TASK-003: Refactor x32_automix to use MixerClient

## Objective
Migrate the gain-sharing automixer to the async `MixerClient` architecture for high-speed meter processing.

## Implementation Details
1. **Meter Subscription**:
   - Subscribe to `/meters/0` (In 1-32 levels).
   - Use `client.subscribe()` to receive and parse the binary meter blob.
2. **Control Loop**:
   - Implement the Dugan-style algorithm as an async task.
   - Calculate gain sharing based on real-time power estimates from meter data.
3. **Gain Application**:
   - Send gain corrections to `/-stat/userpar/XX/value` or relevant fader paths via `client.send_message`.
   - Implement a deadband (0.1dB) to reduce network traffic.

## Success Criteria
- [ ] Automixer maintains stable gain levels across active channels.
- [ ] Processing latency for meter-to-fader loop is < 50ms.
- [ ] Binary `x32_automix` builds and runs without raw `UdpSocket` management.
