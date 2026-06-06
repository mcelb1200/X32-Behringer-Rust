# Open Sound Control (OSC) Parameter Specification and Inheritance

This project uses a zero-allocation parameter parser compiled from declarative JSON specification databases. This system ensures type safety, range limits, and model correctness across the Behringer X32 and XAir series.

---

## 1. Specification Files Architecture

### 1.1 Base Specification (`osc_channels.json`)
The [osc_channels.json](file:///home/pa-system/github/X32-Behringer-Rust/docs/osc_channels.json) file contains the base parameter path patterns, types, and ranges. **The X32 digital mixer serves as the base model** for all parameter definitions.

Example parameter specification:
```json
{
  "pattern": "/ch/{ch}/mix/fader",
  "type": "level"
}
```

### 1.2 Model Overrides (`{model}_overrides.json`)
To support different models without duplicating the base schema, the compiler processes a chain of model overrides:
*   **X32**: Base spec (no overrides).
*   **XR18**: Inherits from **X32**, applies [xr18_overrides.json](file:///home/pa-system/github/X32-Behringer-Rust/docs/xr18_overrides.json).
*   **XR16**: Inherits from **XR18**, applies [xr16_overrides.json](file:///home/pa-system/github/X32-Behringer-Rust/docs/xr16_overrides.json).
*   **XR12**: Inherits from **XR16**, applies [xr12_overrides.json](file:///home/pa-system/github/X32-Behringer-Rust/docs/xr12_overrides.json).

An overrides file can specify three actions:
```json
{
  "remove": [
    "/config/mono/mode"
  ],
  "modify": [
    {
      "pattern": "/config/solo/source",
      "type": "enum",
      "range": ["OFF", "LR", "LRPFL"]
    }
  ],
  "add": [
    {
      "pattern": "/config/custom_feature",
      "type": "enum",
      "range": ["OFF", "ON"]
    }
  ]
}
```

### 1.3 Model Capabilities (`mixer_models.json`)
Channel, bus, and EQ band indices are constrained per model using [mixer_models.json](file:///home/pa-system/github/X32-Behringer-Rust/docs/mixer_models.json). The compiler embeds these limits to dynamically validate wildcard bounds (e.g. `{ch}`, `{bus}`) during path pattern matching.

---

## 2. Developer Guide (Compilation & Code Gen)

1.  **Inheritance Resolution**: During compilation, the `x32_fxparse` crate's [build.rs](file:///home/pa-system/github/X32-Behringer-Rust/libs/x32_fxparse/build.rs) reads the base specs and resolves overrides sequentially along the inheritance chain.
2.  **Code Generation**: For each model, the build script generates a model-specific matcher rust file under `OUT_DIR` (e.g. `channel_parameters_xr18_gen.rs`).
3.  **Parser Integration**: The `parse_parameter` function in `lib.rs` matches the runtime `MixerModel` and includes the corresponding generated matcher at compile time:
    ```rust
    match model {
        MixerModel::X32 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_x32_gen.rs"));
        }
        MixerModel::XR18 => {
            include!(concat!(env!("OUT_DIR"), "/channel_parameters_xr18_gen.rs"));
        }
        // ...
    }
    ```
