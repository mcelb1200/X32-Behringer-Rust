import json
import sys

C_EFFECTS = {
    "HALL": "ffffffffffff",
    "AMBI": "ffffffffff",
    "RPLT": "ffffffffffffffff",
    "ROOM": "ffffffffffffffff",
    "CHAM": "ffffffffffffffff",
    "PLAT": "ffffffffffff",
    "VREV": "fffiifffff",
    "VRM": "ffffffffffffi",
    "GATE": "ffffffffff",
    "RVRS": "fffffffff",
    "DLY": "ffiiifffffff",
    "3TAP": "ffffffiffiffiii",
    "4TAP": "ffffffifififiii",
    "CRS": "fffffffffff",
    "FLNG": "ffffffffffff",
    "PHAS": "ffffffffffff",
    "DIMC": "iiiiiii",
    "FILT": "ffffififffffii",
    "ROTA": "ffffffii",
    "PAN": "fffffffff",
    "SUB": "iifffiifff",
    "D/RV": "fiffffffffff",
    "CR/R": "ffffffffffff",
    "FL/R": "ffffffffffff",
    "D/CR": "fiffffffffff",
    "D/FL": "fiffffffffff",
    "MODD": "fifffffiiffff",
    "GEQ2": "f" * 64,
    "GEQ": "f" * 32,
    "TEQ2": "f" * 64,
    "TEQ": "f" * 32,
    "DES2": "ffffii",
    "DES": "ffffii",
    "P1A": "iffifffifii",
    "P1A2": "iffifffifiiiffifffifii",
    "PQ5": "ififififi",
    "PQ5S": "ififififiififififi",
    "WAVD": "ffffff",
    "LIM": "ffffffii",
    "CMB": "iifffififiiffffiffiffiffiffii",
    "CMB2": "iifffififiiffffiffiffiffiffiiiifffififiiffffiffiffiffiffii",
    "FAC": "iffffff",
    "FAC1M": "iffffffiffffff",
    "FAC2": "iffffffiffffff",
    "LEC": "iffif",
    "LEC2": "iffififfif",
    "ULC": "iffffi",
    "ULC2": "iffffiiffffi",
    "ENH2": "ffffffffiffffffffi",
    "ENH": "ffffffffi",
    "EXC2": "ffffffiffffffi",
    "EXC": "ffffffi",
    "IMG": "fffffff",
    "EDI": "iiifffff",
    "SON": "ifffifff",
    "AMP2": "ffffffffiffffffffi",
    "AMP": "ffffffffi",
    "DRV2": "ffffffffffffffffffff",
    "DRV": "ffffffffff",
    "PIT2": "ffffffffffff",
    "PIT": "ffffff"
}

def main():
    json_path = "/home/pa-system/github/X32-Behringer-Rust/docs/osc_effects.json"
    with open(json_path, 'r') as f:
        js = json.load(f)

    errors = []

    # 1. Check all C_EFFECTS are present in JSON
    json_effects = {eff["effect"]: eff for eff in js}
    for code, expected_fmt in C_EFFECTS.items():
        if code not in json_effects:
            errors.append(f"Effect {code} is missing from JSON")
            continue

        eff = json_effects[code]
        params = eff["parameters"]

        # 2. Check type index matches C enum order
        # We find index in list
        expected_idx = list(C_EFFECTS.keys()).index(code)
        if eff["type_index"] != expected_idx:
            errors.append(f"Effect {code}: type_index mismatch. Expected {expected_idx}, got {eff['type_index']}")

        # 3. Check number of parameters matches the format string length
        expected_len = len(expected_fmt)
        actual_len = len(params)
        if actual_len != expected_len:
            errors.append(f"Effect {code}: Parameter count mismatch. Expected {expected_len}, got {actual_len}")
            continue

        # 4. Check parameter types (f -> linf/logf, i -> enum)
        for idx in range(expected_len):
            par_key = f"par/{idx+1:02d}"
            c_char = expected_fmt[idx]
            p_val = params[par_key]

            p_type = p_val["type"]
            if c_char == 'f' and p_type not in ('linf', 'logf'):
                errors.append(f"Effect {code} {par_key}: Type mismatch. Format expects float, got {p_type}")
            elif c_char == 'i' and p_type != 'enum':
                errors.append(f"Effect {code} {par_key}: Type mismatch. Format expects enum/int, got {p_type}")

            # 5. Check ranges
            p_range = p_val["range"]
            if p_type == "enum":
                if not isinstance(p_range, list) or not all(isinstance(x, str) for x in p_range):
                    errors.append(f"Effect {code} {par_key}: Enum range must be list of strings, got {p_range}")
            else:
                if not isinstance(p_range, list) or len(p_range) != 2 or not all(isinstance(x, (int, float)) for x in p_range):
                    errors.append(f"Effect {code} {par_key}: Numeric range must be [min, max] list, got {p_range}")

    # Check specific parameter values for high-risk parameters
    # E.g. HALL par/01 (Pre Delay)
    hall_p = json_effects["HALL"]["parameters"]
    if hall_p["par/01"]["name"] != "Pre Delay" or hall_p["par/01"]["range"] != [0, 200] or hall_p["par/01"]["type"] != "linf":
        errors.append(f"HALL par/01 (Pre Delay) mismatch: {hall_p['par/01']}")

    if hall_p["par/02"]["name"] != "Decay" or hall_p["par/02"]["range"] != [0.2, 5.0] or hall_p["par/02"]["type"] != "logf":
        errors.append(f"HALL par/02 (Decay) mismatch: {hall_p['par/02']}")

    vrev_p = json_effects["VREV"]["parameters"]
    if vrev_p["par/04"]["name"] != "Vintage" or vrev_p["par/04"]["type"] != "enum" or vrev_p["par/04"]["range"] != ["OFF", "ON"]:
        errors.append(f"VREV par/04 (Vintage) mismatch: {vrev_p['par/04']}")

    if errors:
        print("Verification failed with errors:")
        for err in errors:
            print(f" - {err}")
        sys.exit(1)
    else:
        print("All JSON parameters match C specifications perfectly!")
        sys.exit(0)

if __name__ == '__main__':
    main()
