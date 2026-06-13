use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct ParameterSpec {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
    range: serde_json::Value,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct EffectSpec {
    effect: String,
    name: String,
    type_index: usize,
    parameters: BTreeMap<String, ParameterSpec>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
struct ChannelParamSpec {
    pattern: String,
    #[serde(rename = "type")]
    param_type: String,
    range: Option<serde_json::Value>,
    params: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct OverrideSpec {
    remove: Vec<String>,
    modify: Vec<ChannelParamSpec>,
    add: Vec<ChannelParamSpec>,
}

fn apply_overrides(
    model: &str,
    mut specs: BTreeMap<String, ChannelParamSpec>,
    root_dir: &Path,
) -> BTreeMap<String, ChannelParamSpec> {
    let filename = format!("docs/{}_overrides.json", model.to_lowercase());
    let path = root_dir.join(filename);
    if !path.exists() {
        return specs;
    }
    let file = File::open(&path)
        .unwrap_or_else(|e| panic!("Failed to open overrides {}: {}", path.display(), e));
    let overrides: OverrideSpec = serde_json::from_reader(file)
        .unwrap_or_else(|e| panic!("Failed to parse overrides {}: {}", path.display(), e));

    for pattern in overrides.remove {
        specs.remove(&pattern);
    }
    for spec in overrides.modify {
        specs.insert(spec.pattern.clone(), spec);
    }
    for spec in overrides.add {
        specs.insert(spec.pattern.clone(), spec);
    }
    specs
}

fn resolve_specs_for_model(model: &str, root_dir: &Path) -> Vec<ChannelParamSpec> {
    let base_path = root_dir.join("docs/osc_channels.json");
    let base_file = File::open(&base_path)
        .unwrap_or_else(|e| panic!("Failed to open base channels spec: {}", e));
    let base_vec: Vec<ChannelParamSpec> = serde_json::from_reader(base_file)
        .unwrap_or_else(|e| panic!("Failed to parse base channels: {}", e));

    let mut specs = BTreeMap::new();
    for spec in base_vec {
        specs.insert(spec.pattern.clone(), spec);
    }

    if model == "X32" {
        return specs.into_values().collect();
    }

    // XR18 inherits from X32
    let xr18_specs = apply_overrides("XR18", specs, root_dir);
    if model == "XR18" {
        return xr18_specs.into_values().collect();
    }

    // XR16 inherits from XR18
    let xr16_specs = apply_overrides("XR16", xr18_specs, root_dir);
    if model == "XR16" {
        return xr16_specs.into_values().collect();
    }

    // XR12 inherits from XR16
    let xr12_specs = apply_overrides("XR12", xr16_specs, root_dir);
    xr12_specs.into_values().collect()
}

fn main() {
    println!("cargo:rerun-if-changed=../../docs/osc_effects.json");
    println!("cargo:rerun-if-changed=../../docs/osc_channels.json");
    println!("cargo:rerun-if-changed=../../docs/mixer_models.json");
    println!("cargo:rerun-if-changed=../../docs/xr18_overrides.json");
    println!("cargo:rerun-if-changed=../../docs/xr16_overrides.json");
    println!("cargo:rerun-if-changed=../../docs/xr12_overrides.json");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = Path::new(&manifest_dir).join("../..");

    // 1. Generate FX parameter parser match arms
    let fx_json_path = root_dir.join("docs/osc_effects.json");
    let fx_json_file = File::open(&fx_json_path).expect("Failed to open osc_effects.json");
    let effects: Vec<EffectSpec> =
        serde_json::from_reader(fx_json_file).expect("Failed to parse FX JSON");

    let out_dir = env::var("OUT_DIR").unwrap();
    let fx_dest_path = Path::new(&out_dir).join("fx_parameters_gen.rs");
    let mut fx_out = File::create(&fx_dest_path).expect("Failed to create fx destination file");

    writeln!(fx_out, "match ifx {{").unwrap();
    for eff in effects {
        writeln!(fx_out, "    {} => {{ // {}", eff.type_index, eff.name).unwrap();
        for param in eff.parameters.values() {
            if param.param_type == "linf" {
                let range_arr = param.range.as_array().expect("linf range must be array");
                let min = range_arr[0].as_f64().expect("min must be f64") as f32;
                let max = range_arr[1].as_f64().expect("max must be f64") as f32;
                writeln!(fx_out, "        let val = parse_float(parts.next());").unwrap();
                if min == 0.0 {
                    writeln!(
                        fx_out,
                        "        args.push(OscArg::Float(ratio2float(val, {:.7}))); // {}",
                        max, param.name
                    )
                    .unwrap();
                } else {
                    writeln!(
                        fx_out,
                        "        args.push(OscArg::Float(afine2float(val, {:.7}, {:.7}))); // {}",
                        min,
                        max - min,
                        param.name
                    )
                    .unwrap();
                }
            } else if param.param_type == "logf" {
                let range_arr = param.range.as_array().expect("logf range must be array");
                let min = range_arr[0].as_f64().expect("min must be f64") as f32;
                let max = range_arr[1].as_f64().expect("max must be f64") as f32;
                let log_range = (max / min).ln();
                writeln!(fx_out, "        let val = parse_float(parts.next());").unwrap();
                writeln!(
                    fx_out,
                    "        args.push(OscArg::Float(log2float(val, {:.7}, {:.15}))); // {}",
                    min, log_range, param.name
                )
                .unwrap();
            } else if param.param_type == "enum" {
                let range_arr = param.range.as_array().expect("enum range must be array");
                let items: Vec<String> = range_arr
                    .iter()
                    .map(|v| format!("\"{}\"", v.as_str().expect("enum item must be string")))
                    .collect();
                let items_str = items.join(", ");
                writeln!(fx_out, "        let val = parts.next();").unwrap();
                writeln!(
                    fx_out,
                    "        args.push(OscArg::Int(parse_enum_fx(val, &[ {} ]))); // {}",
                    items_str, param.name
                )
                .unwrap();
            }
        }
        writeln!(fx_out, "    }}").unwrap();
    }
    writeln!(fx_out, "    _ => return None,").unwrap();
    writeln!(fx_out, "}}").unwrap();

    // 2. Generate model-specific channel/bus parameter parser
    let models = vec!["X32", "XR18", "XR16", "XR12"];
    for model in models {
        let channels_specs = resolve_specs_for_model(model, &root_dir);
        let filename = format!("channel_parameters_{}_gen.rs", model.to_lowercase());
        let dest_path = Path::new(&out_dir).join(&filename);
        let mut chan_out = File::create(&dest_path).unwrap_or_else(|e| {
            panic!(
                "Failed to create channels destination file {}: {}",
                filename, e
            )
        });

        writeln!(chan_out, "match parts.len() {{").unwrap();

        let mut grouped: BTreeMap<usize, Vec<ChannelParamSpec>> = BTreeMap::new();
        for spec in channels_specs {
            let parts_len = spec.pattern.trim_start_matches('/').split('/').count();
            grouped.entry(parts_len).or_default().push(spec);
        }

        for (parts_len, specs) in grouped {
            writeln!(chan_out, "    {} => {{", parts_len).unwrap();
            for spec in specs {
                writeln!(chan_out, "        // Pattern: {}", spec.pattern).unwrap();
                let parts: Vec<&str> = spec.pattern.trim_start_matches('/').split('/').collect();
                let mut indent = "        ".to_string();
                let mut if_conditions = Vec::new();

                for (idx, part) in parts.iter().enumerate() {
                    if part.starts_with('{') && part.ends_with('}') {
                        let var_name = &part[1..part.len() - 1];
                        match var_name {
                            "ch" => {
                                if_conditions.push((format!("if let Ok(ch) = parts[{}].parse::<usize>() {{ if ch >= 1 && ch <= limits.channels {{", idx), 2));
                            }
                            "auxin" => {
                                if_conditions.push((format!("if let Ok(aux) = parts[{}].parse::<usize>() {{ if aux >= 1 && aux <= limits.auxins {{", idx), 2));
                            }
                            "fxrtn" => {
                                if_conditions.push((format!("if let Ok(rtn) = parts[{}].parse::<usize>() {{ if rtn >= 1 && rtn <= limits.fxrtns {{", idx), 2));
                            }
                            "bus" => {
                                if_conditions.push((format!("if let Ok(bus) = parts[{}].parse::<usize>() {{ if bus >= 1 && bus <= limits.buses {{", idx), 2));
                            }
                            "mtx" => {
                                if_conditions.push((format!("if let Ok(mtx) = parts[{}].parse::<usize>() {{ if mtx >= 1 && mtx <= limits.matrices {{", idx), 2));
                            }
                            "dca" => {
                                if_conditions.push((format!("if let Ok(dca) = parts[{}].parse::<usize>() {{ if dca >= 1 && dca <= limits.dcas {{", idx), 2));
                            }
                            "fx" => {
                                if_conditions.push((format!("if let Ok(fx) = parts[{}].parse::<usize>() {{ if fx >= 1 && fx <= limits.fx_slots {{", idx), 2));
                            }
                            "headamp" => {
                                if_conditions.push((format!("if let Ok(ha) = parts[{}].parse::<usize>() {{ if ha >= 1 && ha <= limits.channels.max(32) {{", idx), 2));
                            }
                            "mute" => {
                                if_conditions.push((format!("if let Ok(mute) = parts[{}].parse::<usize>() {{ if mute >= 1 && mute <= 6 {{", idx), 2));
                            }
                            "band" => {
                                let band_limit = match parts[0] {
                                    "ch" => "limits.channel_eq_bands",
                                    "auxin" => "limits.auxin_eq_bands",
                                    "fxrtn" => "limits.fxrtn_eq_bands",
                                    "bus" => "limits.bus_eq_bands",
                                    "mtx" => "limits.mtx_eq_bands",
                                    "main" => "limits.main_eq_bands",
                                    _ => "4",
                                };
                                if_conditions.push((format!("if let Ok(band) = parts[{}].parse::<usize>() {{ if band >= 1 && band <= {} {{", idx, band_limit), 2));
                            }
                            "link" => {
                                let link_limit = match parts[1] {
                                    "chlink" => "limits.channels",
                                    "auxlink" => "limits.auxins",
                                    "fxlink" => "limits.fx_slots",
                                    "buslink" => "limits.buses",
                                    "mtxlink" => "limits.matrices",
                                    _ => "32",
                                };
                                if_conditions.push((format!("if let Some((first, second)) = parts[{}].split_once('-') {{ if let (Ok(f), Ok(s)) = (first.parse::<usize>(), second.parse::<usize>()) {{ if f + 1 == s && s <= {} && f % 2 == 1 {{", idx, link_limit), 3));
                            }
                            "param" => {
                                let plist = spec
                                    .params
                                    .as_ref()
                                    .expect("params option must be present for {param}");
                                let list_str = plist
                                    .iter()
                                    .map(|p| format!("\"{}\"", p))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                if_conditions.push((
                                    format!(
                                        "if [{ }].contains(&parts[{}]) {{ if true {{",
                                        list_str, idx
                                    ),
                                    2,
                                ));
                            }
                            _ => {
                                panic!("Unknown wildcard type: {}", var_name);
                            }
                        }
                    } else {
                        if_conditions.push((
                            format!("if parts[{}] == \"{}\" {{ if true {{", idx, part),
                            2,
                        ));
                    }
                }

                let mut braces_to_close = 0;
                for (cond, braces) in &if_conditions {
                    writeln!(chan_out, "{}{}", indent, cond).unwrap();
                    indent.push_str("    ");
                    braces_to_close += braces;
                }

                match spec.param_type.as_str() {
                    "string" => {
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_string(arg_str) {{",
                            indent
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    "enum" => {
                        let range_arr = spec
                            .range
                            .as_ref()
                            .expect("enum spec must have range")
                            .as_array()
                            .expect("enum range must be array");
                        let items: Vec<String> = range_arr
                            .iter()
                            .map(|v| {
                                format!("\"{}\"", v.as_str().expect("enum item must be string"))
                            })
                            .collect();
                        let items_str = items.join(", ");
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_enum(arg_str, &[ {} ]) {{",
                            indent, items_str
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    "level" => {
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_level(arg_str) {{",
                            indent
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    "frequency" => {
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_frequency(arg_str) {{",
                            indent
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    "flin" => {
                        let range_arr = spec
                            .range
                            .as_ref()
                            .expect("flin spec must have range")
                            .as_array()
                            .expect("flin range must be array");
                        let min = range_arr[0].as_f64().expect("min must be f64") as f32;
                        let max = range_arr[1].as_f64().expect("max must be f64") as f32;
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_flin(arg_str, {:.7}, {:.7}) {{",
                            indent, min, max
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    "logf" => {
                        let range_arr = spec
                            .range
                            .as_ref()
                            .expect("logf spec must have range")
                            .as_array()
                            .expect("logf range must be array");
                        let min = range_arr[0].as_f64().expect("min must be f64") as f32;
                        let max = range_arr[1].as_f64().expect("max must be f64") as f32;
                        writeln!(
                            chan_out,
                            "{}if let Some(arg) = parse_logf(arg_str, {:.7}, {:.7}) {{",
                            indent, min, max
                        )
                        .unwrap();
                        writeln!(
                            chan_out,
                            "{}    return Some(OscMessage::new(path.to_string(), vec![arg]));",
                            indent
                        )
                        .unwrap();
                        writeln!(chan_out, "{}}}", indent).unwrap();
                    }
                    _ => panic!("Unknown param type: {}", spec.param_type),
                }

                for (_, braces) in if_conditions.iter().rev() {
                    indent.truncate(indent.len() - 4);
                    writeln!(chan_out, "{}{}", indent, "}".repeat(*braces)).unwrap();
                }
            }
            writeln!(chan_out, "    }}").unwrap();
        }
        writeln!(chan_out, "    _ => {{}}").unwrap();
        writeln!(chan_out, "}}").unwrap();
    }
}
