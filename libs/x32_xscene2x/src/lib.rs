use x32_xsprint::{XsprintValue, xsprint};

pub struct Xscene2xParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Xscene2xParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    pub fn read_token(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.input.len() && !self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
        std::str::from_utf8(&self.input[start..self.pos]).ok()
    }

    pub fn read_quoted_string(&mut self) -> Option<&'a str> {
        while self.pos < self.input.len() && self.input[self.pos] != b'"' {
            self.pos += 1;
        }
        if self.pos >= self.input.len() {
            return None;
        }
        self.pos += 1; // skip first '"'
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != b'"' {
            self.pos += 1;
        }
        let end = self.pos;
        if self.pos < self.input.len() {
            self.pos += 1; // skip second '"'
        }
        std::str::from_utf8(&self.input[start..end]).ok()
    }

    pub fn xp_off_on(&mut self, buf: &mut Vec<u8>, crit_0: &str) {
        if let Some(token) = self.read_token() {
            let ival = if token == crit_0 { 0 } else { 1 };
            xsprint(buf, 's', XsprintValue::String(",i"));
            xsprint(buf, 'i', XsprintValue::Int(ival));
        }
    }

    pub fn xr_float(&mut self) -> Option<f32> {
        let token = self.read_token()?;
        if let Some(k_idx) = token.find('k') {
            let (ival_str, idec_str) = token.split_at(k_idx);
            let idec_str = &idec_str[1..]; // skip 'k'

            let ival: i32 = if ival_str.is_empty() {
                0
            } else {
                ival_str.parse().unwrap_or(0)
            };
            let idec: i32 = if idec_str.is_empty() {
                0
            } else {
                idec_str.parse().unwrap_or(0)
            };

            let mut fval = (ival as f32) * 1000.0;
            let l_i = idec_str.len() + 1;
            if l_i == 2 {
                fval += (idec as f32) * 100.0;
            } else if l_i == 3 {
                fval += (idec as f32) * 10.0;
            } else if l_i == 4 {
                fval += idec as f32;
            }
            Some(fval)
        } else {
            token.parse::<f32>().ok()
        }
    }

    pub fn xp_percent(&mut self, buf: &mut Vec<u8>) {
        if let Some(token) = self.read_token() {
            if let Ok(mut fval) = token.parse::<f32>() {
                fval /= 100.0;
                xsprint(buf, 's', XsprintValue::String(",f"));
                xsprint(buf, 'f', XsprintValue::Float(fval));
            }
        }
    }

    pub fn xp_linf(&mut self, buf: &mut Vec<u8>, xmin: f32, lmaxmin: f32, mut xstep: f32) {
        if let Some(mut fval) = self.xr_float() {
            fval = (fval - xmin) / lmaxmin;
            xstep = lmaxmin / xstep;
            fval = (fval * xstep).round() / xstep;
            if fval <= 0.0 {
                fval = 0.0;
            }
            if fval > 1.0 {
                fval = 1.0;
            }
            xsprint(buf, 's', XsprintValue::String(",f"));
            xsprint(buf, 'f', XsprintValue::Float(fval));
        }
    }

    pub fn xp_logf(&mut self, buf: &mut Vec<u8>, xmin: f32, lmaxmin: f32, nsteps: i32) {
        if let Some(mut fval) = self.xr_float() {
            fval = (fval / xmin).ln() / lmaxmin;
            fval = (fval * (nsteps as f32)).round() / (nsteps as f32);
            if fval <= 0.0 {
                fval = 0.0;
            }
            if fval > 1.0 {
                fval = 1.0;
            }
            xsprint(buf, 's', XsprintValue::String(",f"));
            xsprint(buf, 'f', XsprintValue::Float(fval));
        }
    }

    pub fn xp_int(&mut self, buf: &mut Vec<u8>) {
        if let Some(token) = self.read_token() {
            if let Ok(ival) = token.parse::<i32>() {
                xsprint(buf, 's', XsprintValue::String(",i"));
                xsprint(buf, 'i', XsprintValue::Int(ival));
            }
        }
    }

    pub fn xp_str(&mut self, buf: &mut Vec<u8>) {
        if let Some(s) = self.read_quoted_string() {
            xsprint(buf, 's', XsprintValue::String(",s"));
            xsprint(buf, 's', XsprintValue::String(s));
        }
    }

    pub fn xp_list(&mut self, buf: &mut Vec<u8>, list: &[&str]) {
        if let Some(token) = self.read_token() {
            if let Some(ival) = list.iter().position(|&s| s == token) {
                xsprint(buf, 's', XsprintValue::String(",i"));
                xsprint(buf, 'i', XsprintValue::Int(ival as i32));
            }
        }
    }

    pub fn xp_fxlist(&mut self, buf: &mut Vec<u8>, list: &[&str]) -> Option<i32> {
        if let Some(token) = self.read_token() {
            if let Some(ival) = list.iter().position(|&s| s == token) {
                let ival = ival as i32;
                xsprint(buf, 's', XsprintValue::String(",i"));
                xsprint(buf, 'i', XsprintValue::Int(ival));
                return Some(ival);
            }
        }
        None
    }

    pub fn xp_bit(&mut self, buf: &mut Vec<u8>) {
        if let Some(token) = self.read_token() {
            if let Some(pct_idx) = token.find('%') {
                let bit_str = &token[pct_idx + 1..];
                let mut ival = 0;
                for (j, ch) in bit_str.chars().rev().enumerate() {
                    if ch == '1' {
                        ival |= 1 << j;
                    }
                }
                xsprint(buf, 's', XsprintValue::String(",i"));
                xsprint(buf, 'i', XsprintValue::Int(ival));
            }
        }
    }

    pub fn xp_frequency(&mut self, buf: &mut Vec<u8>, nsteps: i32) {
        if let Some(mut fval) = self.xr_float() {
            fval = (fval / 20.0).ln() / 6.907755279;
            fval = (fval * (nsteps as f32)).round() / (nsteps as f32);
            if fval <= 0.0 {
                fval = 0.0;
            }
            if fval > 1.0 {
                fval = 1.0;
            }
            xsprint(buf, 's', XsprintValue::String(",f"));
            xsprint(buf, 'f', XsprintValue::Float(fval));
        }
    }

    pub fn xp_level(&mut self, buf: &mut Vec<u8>, nsteps: i32) {
        if let Some(token) = self.read_token() {
            let fval = if token.starts_with("-oo") {
                0.0
            } else if let Ok(mut v) = token.parse::<f32>() {
                let n = nsteps as f32;
                if v < -60.0 {
                    v = v * 0.00208333333 + 0.1875;
                    v = (v * n).round() / n;
                    if v < 0.0 {
                        v = 0.0;
                    }
                    v
                } else if v < -30.0 {
                    v = v * 0.00625 + 0.4375;
                    v = (v * n).round() / n;
                    v
                } else if v < -10.0 {
                    v = v * 0.0125 + 0.625;
                    v = (v * n).round() / n;
                    v
                } else if v <= 10.0 {
                    v = v * 0.025 + 0.75;
                    if v > 1.0 {
                        v = 1.0;
                    }
                    v
                } else {
                    v
                }
            } else {
                return;
            };

            xsprint(buf, 's', XsprintValue::String(",f"));
            xsprint(buf, 'f', XsprintValue::Float(fval));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_off_on() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("ON OFF ON");
        parser.xp_off_on(&mut buf, "OFF");
        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);

        buf.clear();
        parser.xp_off_on(&mut buf, "OFF");
        let mut expected2 = b",i\0\0".to_vec();
        expected2.extend_from_slice(&0i32.to_be_bytes());
        assert_eq!(buf, expected2);
    }

    #[test]
    fn test_xr_float() {
        let mut parser = Xscene2xParser::new("12.34 5k6 7k 0k12");
        assert_eq!(parser.xr_float(), Some(12.34));
        assert_eq!(parser.xr_float(), Some(5600.0));
        assert_eq!(parser.xr_float(), Some(7000.0));
        assert_eq!(parser.xr_float(), Some(120.0));
    }

    #[test]
    fn test_xp_percent() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("50.0");
        parser.xp_percent(&mut buf);
        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&0.5f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_linf() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("5.0");
        parser.xp_linf(&mut buf, 0.0, 10.0, 10.0);
        let mut expected = b",f\0\0".to_vec();

        // 5.0 -> (5 - 0)/10 = 0.5
        // xstep = 10 / 10 = 1.0
        // fval = round(0.5 * 1.0) / 1.0 = round(0.5) / 1.0 = 1.0
        expected.extend_from_slice(&1.0f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_logf() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("316.227766");
        // log10(316.22) / log10(1000) ~ 2.5 / 3.0
        // Wait, log(x / xmin) / lmaxmin
        parser.xp_logf(&mut buf, 10.0, (1000.0f32).ln(), 100);

        let fval = ((316.227766f32 / 10.0).ln() / (1000.0f32).ln() * 100.0).round() / 100.0;
        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&fval.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_int() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("42");
        parser.xp_int(&mut buf);
        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&42i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_str() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new(r#"  "hello world" "#);
        parser.xp_str(&mut buf);
        let mut expected = b",s\0\0".to_vec();
        expected.extend_from_slice(b"hello world\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_list() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("B");
        parser.xp_list(&mut buf, &["A", "B", "C"]);
        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_fxlist() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("C");
        let res = parser.xp_fxlist(&mut buf, &["A", "B", "C"]);
        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&2i32.to_be_bytes());
        assert_eq!(buf, expected);
        assert_eq!(res, Some(2));
    }

    #[test]
    fn test_xp_bit() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("%101");
        parser.xp_bit(&mut buf);
        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&5i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_frequency() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("1000.0");
        parser.xp_frequency(&mut buf, 201);

        let mut expected = b",f\0\0".to_vec();
        let mut fval = (1000.0f32 / 20.0).ln() / 6.907755279;
        fval = (fval * 201.0).round() / 201.0;
        expected.extend_from_slice(&fval.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_level_infinity() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("-oo");
        parser.xp_level(&mut buf, 1023);
        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&0.0f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xp_level_positive() {
        let mut buf = Vec::new();
        let mut parser = Xscene2xParser::new("0.0");
        parser.xp_level(&mut buf, 1023);
        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&0.75f32.to_be_bytes());
        assert_eq!(buf, expected);
    }
}
