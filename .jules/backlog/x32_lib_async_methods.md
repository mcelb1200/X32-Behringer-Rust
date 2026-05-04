# Task: Update x32_lib with Async MixerClient Methods

## Context
`x32_lib` now contains a `MixerClient` which is the preferred way to communicate with the mixer. However, the existing top-level functions in `lib.rs` (like `set_parameter`, `get_parameter`) are still based on raw `UdpSocket`.

## Goal
Add async versions of common library functions that accept a `&MixerClient` instead of a `&UdpSocket`.

## Required Components & Logic

### 1. Async Parameter Methods
- **Implement** `async fn set_parameter_async(client: &MixerClient, address: &str, value: f32) -> Result<()>`.
- **Implement** `async fn get_parameter_async(client: &MixerClient, address: &str) -> Result<f32>`.
- **Implement** `async fn query_value_async(client: &MixerClient, address: &str) -> Result<OscArg>`.

### 2. Scene Parsing Integration
- **Update** `scene_parse.rs` logic to optionally work with `MixerClient` if beneficial for live scene retrieval.

### 3. Error Handling
- **Ensure** all new async methods use the centralized `Result` type and provide descriptive errors for timeouts or network failures.

## Constraints
- **Maintain** the existing sync functions for backward compatibility where needed (though they should eventually be deprecated).
- Ensure the methods are well-documented with examples.
