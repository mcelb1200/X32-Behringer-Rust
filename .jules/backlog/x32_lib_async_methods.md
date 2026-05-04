# TASK-001: Update x32_lib with Async MixerClient Methods

## Objective
Provide a unified async API in `x32_lib::lib.rs` that works directly with `MixerClient`.

## Implementation Details
1. **Parameter Management**:
   ```rust
   pub async fn set_parameter_async(client: &MixerClient, address: &str, value: f32) -> Result<()>
   pub async fn get_parameter_async(client: &MixerClient, address: &str) -> Result<f32>
   ```
2. **Generic Query**:
   ```rust
   pub async fn query_value_async(client: &MixerClient, address: &str) -> Result<OscArg>
   ```
3. **Internal Logic**:
   - Use `client.subscribe()` to wait for specific response paths.
   - Implement bounded timeouts (default 500ms) for all queries.

## Success Criteria
- [ ] New methods exist in `libs/x32_lib/src/lib.rs`.
- [ ] Existing sync methods are marked `#[deprecated]`.
- [ ] Unit tests in `x32_lib` verify async parameter retrieval against a mock/emulator.
