use x32_xsprint::{XsprintValue, xsprint};

pub struct SceneParser<'a> {
    input: &'a str,
}

impl<'a> SceneParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    /// Read next whitespace-delimited token and advance the input slice.
    fn next_token(&mut self) -> Option<&'a str> {
        self.input = self.input.trim_start();
        if self.input.is_empty() {
            return None;
        }

        let end_idx = self
            .input
            .find(|c: char| c.is_whitespace())
            .unwrap_or(self.input.len());
        let token = &self.input[..end_idx];
        self.input = &self.input[end_idx..];
        Some(token)
    }

    pub fn xoff_on(&mut self, buf: &mut Vec<u8>, crit_0: &str) -> usize {
        let token = self.next_token().unwrap_or("");

        let initial_len = buf.len();
        xsprint(buf, 's', XsprintValue::String(",i"));
        let ival = if token == crit_0 { 0 } else { 1 };
        xsprint(buf, 'i', XsprintValue::Int(ival));
        buf.len() - initial_len
    }

    pub fn xr_float(&mut self) -> f32 {
        let token = self.next_token().unwrap_or("0.0");

        // "float number, in the form nnn, nnn.ff, or nnnkff"
        if let Some(k_idx) = token.find('k') {
            let int_part = &token[..k_idx];
            let dec_part = &token[k_idx + 1..];

            let ival = if int_part.is_empty() {
                0
            } else {
                int_part.parse::<i32>().unwrap_or(0)
            };
            let idec = if dec_part.is_empty() {
                0
            } else {
                dec_part.parse::<i32>().unwrap_or(0)
            };

            let mut fval = (ival as f32) * 1000.0;
            let dec_len = dec_part.len();

            if dec_len == 1 {
                fval += (idec as f32) * 100.0;
            } else if dec_len == 2 {
                fval += (idec as f32) * 10.0;
            } else if dec_len == 3 {
                fval += idec as f32;
            } else {
                // Not exactly handled by C code explicitly, but logic dictates:
                let scale = 10i32.pow(3_u32.saturating_sub(dec_len as u32));
                fval += (idec as f32) * (scale as f32);
            }
            return fval;
        }

        token.parse::<f32>().unwrap_or(0.0)
    }

    pub fn xp_percent(&mut self, buf: &mut Vec<u8>) -> usize {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("0.0");
        let fval = token.parse::<f32>().unwrap_or(0.0) / 100.0;

        xsprint(buf, 's', XsprintValue::String(",f"));
        xsprint(buf, 'f', XsprintValue::Float(fval));
        buf.len() - initial_len
    }

    pub fn xp_linf(&mut self, buf: &mut Vec<u8>, xmin: f32, lmaxmin: f32, xstep: f32) -> usize {
        let initial_len = buf.len();
        let mut fval = self.xr_float();

        fval = (fval - xmin) / lmaxmin;
        let xstep_scaled = lmaxmin / xstep;
        fval = (fval * xstep_scaled).round() / xstep_scaled;

        if fval <= 0.0 {
            fval = 0.0;
        }
        if fval > 1.0 {
            fval = 1.0;
        }

        xsprint(buf, 's', XsprintValue::String(",f"));
        xsprint(buf, 'f', XsprintValue::Float(fval));
        buf.len() - initial_len
    }

    pub fn xp_logf(&mut self, buf: &mut Vec<u8>, xmin: f32, lmaxmin: f32, nsteps: i32) -> usize {
        let initial_len = buf.len();
        let mut fval = self.xr_float();

        fval = (fval / xmin).ln() / lmaxmin;
        let nsteps_f = nsteps as f32;
        fval = (fval * nsteps_f).round() / nsteps_f;

        if fval <= 0.0 {
            fval = 0.0;
        }
        if fval > 1.0 {
            fval = 1.0;
        }

        xsprint(buf, 's', XsprintValue::String(",f"));
        xsprint(buf, 'f', XsprintValue::Float(fval));
        buf.len() - initial_len
    }

    pub fn xp_int(&mut self, buf: &mut Vec<u8>) -> usize {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("0");
        let ival = token.parse::<i32>().unwrap_or(0);

        xsprint(buf, 's', XsprintValue::String(",i"));
        xsprint(buf, 'i', XsprintValue::Int(ival));
        buf.len() - initial_len
    }

    pub fn xp_str(&mut self, buf: &mut Vec<u8>) -> usize {
        let initial_len = buf.len();

        // Read string between double quotes
        // We do not use next_token() here because spaces are allowed inside quotes
        let mut str_val = String::new();
        if let Some(start_idx) = self.input.find('"') {
            self.input = &self.input[start_idx + 1..];
            if let Some(end_idx) = self.input.find('"') {
                str_val = self.input[..end_idx].to_string();
                self.input = &self.input[end_idx + 1..];
            } else {
                // No closing quote, take the rest
                str_val = self.input.to_string();
                self.input = "";
            }
        }

        xsprint(buf, 's', XsprintValue::String(",s"));
        xsprint(buf, 's', XsprintValue::String(&str_val));
        buf.len() - initial_len
    }

    pub fn xp_list(&mut self, buf: &mut Vec<u8>, list: &[&str]) -> usize {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("");

        let mut ival = 0;
        for (i, &item) in list.iter().enumerate() {
            if token == item {
                ival = i as i32;
                break;
            }
        }

        xsprint(buf, 's', XsprintValue::String(",i"));
        xsprint(buf, 'i', XsprintValue::Int(ival));
        buf.len() - initial_len
    }

    pub fn xp_fxlist(&mut self, buf: &mut Vec<u8>, list: &[&str]) -> (usize, i32) {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("");

        let mut p_ival = 0;
        for (i, &item) in list.iter().enumerate() {
            if token == item {
                p_ival = i as i32;
                break;
            }
        }

        xsprint(buf, 's', XsprintValue::String(",i"));
        xsprint(buf, 'i', XsprintValue::Int(p_ival));
        (buf.len() - initial_len, p_ival)
    }

    pub fn xp_bit(&mut self, buf: &mut Vec<u8>) -> usize {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("");

        let mut ival = 0;
        let mut j = 0;

        // Iterate backwards until we hit '%' or beginning of token
        for c in token.chars().rev() {
            if c == '%' {
                break;
            }
            if c == '1' {
                ival |= 1 << j;
            }
            j += 1;
        }

        xsprint(buf, 's', XsprintValue::String(",i"));
        xsprint(buf, 'i', XsprintValue::Int(ival));
        buf.len() - initial_len
    }

    pub fn xp_frequency(&mut self, buf: &mut Vec<u8>, nsteps: i32) -> usize {
        let initial_len = buf.len();
        let mut fval = self.xr_float();

        fval = (fval / 20.0).ln() / 6.907755279;
        let nsteps_f = nsteps as f32;
        fval = (fval * nsteps_f).round() / nsteps_f;

        if fval <= 0.0 {
            fval = 0.0;
        }
        if fval > 1.0 {
            fval = 1.0;
        }

        xsprint(buf, 's', XsprintValue::String(",f"));
        xsprint(buf, 'f', XsprintValue::Float(fval));
        buf.len() - initial_len
    }

    pub fn xp_level(&mut self, buf: &mut Vec<u8>, nsteps: i32) -> usize {
        let initial_len = buf.len();
        let token = self.next_token().unwrap_or("");

        let mut fval = 0.0;
        if token == "-oo" || token == "-00" {
            // Accommodate potential typos, C says "-oo"
            fval = 0.0;
        } else if let Ok(val) = token.parse::<f32>() {
            fval = val;
            let nsteps_f = nsteps as f32;
            if fval < -60.0 {
                fval = fval * 0.00208333333 + 0.1875;
                fval = (fval * nsteps_f).round() / nsteps_f;
                if fval < 0.0 {
                    fval = 0.0;
                }
            } else if fval < -30.0 {
                fval = 0.00625 * fval + 0.4375;
                fval = (fval * nsteps_f).round() / nsteps_f;
            } else if fval < -10.0 {
                fval = 0.0125 * fval + 0.625;
                fval = (fval * nsteps_f).round() / nsteps_f;
            } else if fval <= 10.0 {
                fval = fval * 0.025 + 0.75;
                if fval > 1.0 {
                    fval = 1.0;
                }
            }
        }

        xsprint(buf, 's', XsprintValue::String(",f"));
        xsprint(buf, 'f', XsprintValue::Float(fval));
        buf.len() - initial_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoff_on() {
        let mut parser = SceneParser::new("ON OFF");
        let mut buf = Vec::new();

        parser.xoff_on(&mut buf, "OFF"); // First token is "ON" != "OFF" -> 1
        assert_eq!(buf, b",i\0\0\0\0\0\x01"); // ,i padding 2 nulls + int 1 (big endian)

        buf.clear();
        parser.xoff_on(&mut buf, "OFF"); // Second token is "OFF" == "OFF" -> 0
        assert_eq!(buf, b",i\0\0\0\0\0\x00");
    }

    #[test]
    fn test_xr_float() {
        // Standard floats
        assert_eq!(SceneParser::new("123.45").xr_float(), 123.45);
        assert_eq!(SceneParser::new("-10.5").xr_float(), -10.5);

        // "k" floats
        assert_eq!(SceneParser::new("1k2").xr_float(), 1200.0);
        assert_eq!(SceneParser::new("5k05").xr_float(), 5050.0);
        assert_eq!(SceneParser::new("10k").xr_float(), 10000.0);
        assert_eq!(SceneParser::new("k5").xr_float(), 500.0);
    }

    #[test]
    fn test_xp_percent() {
        let mut parser = SceneParser::new("50.0");
        let mut buf = Vec::new();
        parser.xp_percent(&mut buf);

        // ,f followed by 0.5f32 big endian
        assert_eq!(&buf[0..4], b",f\0\0");
        let fval = f32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        assert_eq!(fval, 0.5);
    }

    #[test]
    fn test_xp_str() {
        let mut parser = SceneParser::new(r#" "Hello World" "#);
        let mut buf = Vec::new();
        parser.xp_str(&mut buf);

        assert_eq!(&buf[0..4], b",s\0\0");
        // String format: "Hello World\0" padded to 4 bytes boundary
        // length = 11, +1 null = 12. So 12 bytes.
        let expected_str = b"Hello World\0";
        assert_eq!(&buf[4..16], expected_str);
    }

    #[test]
    fn test_xp_list() {
        let mut parser = SceneParser::new("B A C");
        let mut buf = Vec::new();
        let list = ["A", "B", "C"];

        parser.xp_list(&mut buf, &list); // reads B -> index 1
        assert_eq!(buf, b",i\0\0\0\0\0\x01");

        buf.clear();
        parser.xp_list(&mut buf, &list); // reads A -> index 0
        assert_eq!(buf, b",i\0\0\0\0\0\x00");
    }

    #[test]
    fn test_xp_bit() {
        let mut parser = SceneParser::new("%101");
        let mut buf = Vec::new();
        parser.xp_bit(&mut buf);
        // %101 binary is 5
        assert_eq!(buf, b",i\0\0\0\0\0\x05");
    }

    #[test]
    fn test_xp_level() {
        let mut parser = SceneParser::new("-oo 0.0");
        let mut buf = Vec::new();
        parser.xp_level(&mut buf, 100);

        let fval = f32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        assert_eq!(fval, 0.0);

        buf.clear();
        parser.xp_level(&mut buf, 100); // parses 0.0
        let fval = f32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        assert_eq!(fval, 0.75);
    }
}
