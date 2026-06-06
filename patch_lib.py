import re

with open("libs/x32_lib/src/lib.rs", "r") as f:
    text = f.read()

verify_async = """
/// Verifies if a given FX slot contains a specific effect type asynchronously.
///
/// # Arguments
///
/// * `client` - A `MixerClient` connected to the mixer.
/// * `slot` - The FX slot number (1-8).
/// * `expected_type` - A substring expected to be in the effect type name (e.g., "EQ").
///
/// # Returns
///
/// A `Result` containing a boolean indicating if the effect matches.
pub async fn verify_fx_type_async(client: &MixerClient, slot: u8, expected_type: &str) -> Result<bool> {
    match client.query_node(&format!("fx/{}", slot)).await {
        Ok(res) => Ok(res.contains(expected_type)),
        Err(_) => Ok(false),
    }
}
"""

text = text + "\n" + verify_async

with open("libs/x32_lib/src/lib.rs", "w") as f:
    f.write(text)
