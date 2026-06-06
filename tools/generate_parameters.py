import json
import re

C_EFFECTS = [
    ("HALL", "ffffffffffff"),
    ("AMBI", "ffffffffff"),
    ("RPLT", "ffffffffffffffff"),
    ("ROOM", "ffffffffffffffff"),
    ("CHAM", "ffffffffffffffff"),
    ("PLAT", "ffffffffffff"),
    ("VREV", "fffiifffff"),
    ("VRM", "ffffffffffffi"),
    ("GATE", "ffffffffff"),
    ("RVRS", "fffffffff"),
    ("DLY", "ffiiifffffff"),
    ("3TAP", "ffffffiffiffiii"),
    ("4TAP", "ffffffifififiii"),
    ("CRS", "fffffffffff"),
    ("FLNG", "ffffffffffff"),
    ("PHAS", "ffffffffffff"),
    ("DIMC", "iiiiiii"),
    ("FILT", "ffffififffffii"),
    ("ROTA", "ffffffii"),
    ("PAN", "fffffffff"),
    ("SUB", "iifffiifff"),
    ("D/RV", "fiffffffffff"),
    ("CR/R", "ffffffffffff"),
    ("FL/R", "ffffffffffff"),
    ("D/CR", "fiffffffffff"),
    ("D/FL", "fiffffffffff"),
    ("MODD", "fifffffiiffff"),
    ("GEQ2", "f" * 64),
    ("GEQ", "f" * 32),
    ("TEQ2", "f" * 64),
    ("TEQ", "f" * 32),
    ("DES2", "ffffii"),
    ("DES", "ffffii"),
    ("P1A", "iffifffifii"),
    ("P1A2", "iffifffifiiiffifffifii"),
    ("PQ5", "ififififi"),
    ("PQ5S", "ififififiififififi"),
    ("WAVD", "ffffff"),
    ("LIM", "ffffffii"),
    ("CMB", "iifffififiiffffiffiffiffiffii"),
    ("CMB2", "iifffififiiffffiffiffiffiffiiiifffififiiffffiffiffiffiffii"),
    ("FAC", "iffffff"),
    ("FAC1M", "iffffffiffffff"),
    ("FAC2", "iffffffiffffff"),
    ("LEC", "iffif"),
    ("LEC2", "iffififfif"),
    ("ULC", "iffffi"),
    ("ULC2", "iffffiiffffi"),
    ("ENH2", "ffffffffiffffffffi"),
    ("ENH", "ffffffffi"),
    ("EXC2", "ffffffiffffffi"),
    ("EXC", "ffffffi"),
    ("IMG", "fffffff"),
    ("EDI", "iiifffff"),
    ("SON", "ifffifff"),
    ("AMP2", "ffffffffiffffffffi"),
    ("AMP", "ffffffffi"),
    ("DRV2", "ffffffffffffffffffff"),
    ("DRV", "ffffffffff"),
    ("PIT2", "ffffffffffff"),
    ("PIT", "ffffff")
]

name_map = {
    "HALL": "Hall Reverb", "AMBI": "Ambiance Reverb", "RPLT": "Rich Plate Reverb",
    "ROOM": "Room Reverb", "CHAM": "Chamber Reverb", "PLAT": "Plate Reverb",
    "VREV": "Vintage Reverb", "VRM": "Vintage Room", "GATE": "Gated Reverb",
    "RVRS": "Reverse Reverb", "DLY": "Stereo Delay", "3TAP": "3-Tap Delay",
    "4TAP": "4-Tap Delay", "CRS": "Stereo Chorus", "FLNG": "Stereo Flanger",
    "PHAS": "Stereo Phaser", "DIMC": "Dimensional Chorus", "FILT": "Mood Filter",
    "ROTA": "Rotary Speaker", "PAN": "Tremolo / Panner", "SUB": "Sub Octaver",
    "D/RV": "Delay / Chamber", "CR/R": "Chorus / Chamber", "FL/R": "Flanger / Chamber",
    "D/CR": "Delay / Chorus", "D/FL": "Delay / Flanger", "MODD": "Modulation Delay",
    "GEQ2": "Dual Graphic EQ", "GEQ": "Stereo Graphic EQ", "TEQ2": "Dual True EQ",
    "TEQ": "Stereo True EQ", "DES2": "Dual De-Esser", "DES": "Stereo De-Esser",
    "P1A": "Stereo Program EQ", "P1A2": "Dual Program EQ", "PQ5": "Stereo Midrange EQ",
    "PQ5S": "Dual Midrange EQ", "WAVD": "Wave Designer", "LIM": "Precision Limiter",
    "CMB": "Stereo Combinator", "CMB2": "Dual Combinator", "FAC": "Stereo Fair Compressor",
    "FAC1M": "M/S Fair Compressor", "FAC2": "Dual Fair Compressor", "LEC": "Stereo Leisure Compressor",
    "LEC2": "Dual Leisure Compressor", "ULC": "Stereo Ultimo Compressor", "ULC2": "Dual Ultimo Compressor",
    "ENH2": "Dual Enhancer", "ENH": "Stereo Enhancer", "EXC2": "Dual Exciter",
    "EXC": "Stereo Exciter", "IMG": "Stereo Imager", "EDI": "Edison EX1",
    "SON": "Sound Maxer", "AMP2": "Dual Guitar Amp", "AMP": "Stereo Guitar Amp",
    "DRV2": "Dual Tube Stage", "DRV": "Stereo Tube Stage", "PIT2": "Dual Pitch Shifter",
    "PIT": "Stereo Pitch Shifter"
}

def clean_range_str(r_str):
    return r_str.strip().strip('[]')

def parse_range(ptype, r_str):
    if ptype == "enum":
        items = []
        for x in r_str.split(','):
            x = x.strip()
            if x:
                items.append(x)
        return items
    else:
        parts = re.split(r'…|\.\.\.', r_str)
        if len(parts) != 2:
            parts = re.findall(r'[\-\d\.]+(?:k|K)?', r_str)
        
        def parse_num(s):
            s = s.strip().lower()
            factor = 1.0
            if s.endswith('k'):
                factor = 1000.0
                s = s[:-1]
            return float(s) * factor

        try:
            return [parse_num(parts[0]), parse_num(parts[1])]
        except Exception:
            return [0.0, 1.0]

def parse_md_file(path):
    with open(path, 'r') as f:
        content = f.read()

    start_idx = content.find("\nEFFECTS\n")
    if start_idx == -1:
        start_idx = content.find("EFFECTS")
    
    effects_text = content[start_idx:]
    
    parsed = {}
    
    for code, format_str in C_EFFECTS:
        md_code = code
        if code == "TEQ":
            md_code = "GEQ"
        elif code == "TEQ2":
            md_code = "GEQ2"
        elif code == "WAVD":
            md_code = "WAV"
        elif code == "FAC1M":
            md_code = "FAC2"
            
        pattern = rf'\n({re.escape(md_code)}|_?{re.escape(md_code)})\s+([a-z0-9]+)\s+'
        match = re.search(pattern, effects_text)
        
        if not match:
            pattern = rf'\n({re.escape(md_code)}|_?{re.escape(md_code)})\s+(\d+\s+[a-z0-9]+)\s+'
            match = re.search(pattern, effects_text)
            
        if not match:
            continue
            
        snippet = effects_text[match.end():]
        lines = snippet.split('\n')
        
        expected_count = len(format_str)
        
        raw_params = []
        current_param = None
        
        for line in lines:
            if re.match(r'^[A-Z][a-zA-Z0-9\s\-]{3,30}$', line.strip()) and not any(x in line for x in ["linf", "logf", "enum"]):
                break
                
            m_param = re.search(r'\b(linf|logf|enum)\s+\[([^\]\n]*)\]?', line)
            if m_param:
                if current_param:
                    raw_params.append(current_param)
                
                ptype = m_param.group(1)
                prange = m_param.group(2)
                before_type = line[:m_param.start()].strip()
                name = re.sub(r'^\s*(\([^\)]+\))?\s*', '', before_type).strip()
                
                closed = ']' in line[m_param.start():]
                
                current_param = {
                    "name": name,
                    "type": ptype,
                    "raw_range": prange,
                    "closed": closed
                }
            elif current_param:
                if len(line) > 50:
                    right_part = line[50:].strip()
                    if right_part:
                        if not current_param["closed"]:
                            if ']' in right_part:
                                cont = right_part.split(']')[0].strip()
                                current_param["raw_range"] += " " + cont
                                current_param["closed"] = True
                            else:
                                current_param["raw_range"] += " " + right_part
        
        if current_param:
            raw_params.append(current_param)
            
        expanded_params = []
        for rp in raw_params:
            m_mult = re.match(r'^(\d+)\s*x\s*(.+)$', rp["name"])
            if m_mult:
                count = int(m_mult.group(1))
                base_name = m_mult.group(2).strip()
                for c in range(1, count + 1):
                    expanded_params.append({
                        "name": f"{base_name} {c}",
                        "type": rp["type"],
                        "raw_range": rp["raw_range"]
                    })
            else:
                expanded_params.append(rp)
                
        final_params = {}
        for idx in range(expected_count):
            par_key = f"par/{idx+1:02d}"
            c_char = format_str[idx]
            
            if idx < len(expanded_params):
                ep = expanded_params[idx]
                ptype = ep["type"]
                if c_char == 'f' and ptype == 'enum':
                    ptype = 'linf'
                elif c_char == 'i' and ptype != 'enum':
                    ptype = 'enum'
                
                p_range = parse_range(ptype, ep["raw_range"])
                
                # Apply PLAT decay range override
                if code == "PLAT" and par_key == "par/02":
                    p_range = [0.5, 10.0]
                    
                name = ep["name"]
                if code == "TEQ":
                    name = name.replace("GEQ", "TEQ")
                elif code == "TEQ2":
                    name = name.replace("GEQ2", "TEQ2")
                elif code == "FAC1M":
                    name = name.replace("FAC2", "FAC1M").replace("A", "M").replace("B", "S")
            else:
                ptype = 'linf' if c_char == 'f' else 'enum'
                p_range = [0.0, 1.0] if ptype == 'linf' else ["OFF", "ON"]
                name = f"Unknown {par_key}"
                
            unit = None
            if ptype != 'enum':
                for u in ["ms", "Hz", "dB", "%", "deg"]:
                    if u in name:
                        unit = u
                        break
            
            final_params[par_key] = {
                "name": name,
                "type": ptype,
                "range": p_range
            }
            if unit:
                final_params[par_key]["unit"] = unit
                
        parsed[code] = final_params
        
    return parsed

def main():
    md_path = "/home/pa-system/github/X32-Behringer-Rust/docs/unofficial_x32_osc_remote_protocol.md"
    json_path = "/home/pa-system/github/X32-Behringer-Rust/docs/osc_effects.json"
    
    parsed = parse_md_file(md_path)
    
    output_list = []
    for idx, (code, _) in enumerate(C_EFFECTS):
        if code in parsed:
            output_list.append({
                "effect": code,
                "name": name_map.get(code, code),
                "type_index": idx,
                "parameters": parsed[code]
            })
            
    with open(json_path, 'w') as f:
        json.dump(output_list, f, indent=2)
        
    print(f"Generated {len(output_list)} effects into {json_path}")

if __name__ == '__main__':
    main()
