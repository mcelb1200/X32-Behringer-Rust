use std::io::{self, BufRead};
use x32_xsprint::{xsprint, XsprintValue};

/// Ported from `XOff_On(char *buf, int k, char* Crit_0)`
pub fn x_off_on<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    crit_0: &str,
) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    xsprint(buf, 's', XsprintValue::String(",i"));
    let ival = if word == crit_0 { 0 } else { 1 };
    xsprint(buf, 'i', XsprintValue::Int(ival));

    Ok(buf.len())
}

/// Ported from `Xr_float()`
pub fn x_r_float<R: BufRead>(reader: &mut R) -> io::Result<f32> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    if word.is_empty() {
        return Ok(0.0);
    }

    if word.contains('.') {
        return Ok(word.parse::<f32>().unwrap_or(0.0));
    } else if let Some(idx) = word.find('k') {
        let int_part = &word[..idx];
        let dec_part = &word[idx + 1..];

        let mut ival: i32 = 0;
        let mut idec: i32 = 0;
        if !int_part.is_empty() {
            ival = int_part.parse::<i32>().unwrap_or(0);
        }
        if !dec_part.is_empty() {
            idec = dec_part.parse::<i32>().unwrap_or(0);
        }

        let mut fval = ival as f32 * 1000.0;
        let l = word.len();
        if l - idx == 2 {
            fval += idec as f32 * 100.0;
        } else if l - idx == 3 {
            fval += idec as f32 * 10.0;
        } else if l - idx == 4 {
            fval += idec as f32;
        }
        return Ok(fval);
    }

    Ok(word.parse::<f32>().unwrap_or(0.0))
}

/// Ported from `Xp_percent(char *buf, int k)`
pub fn x_p_percent<R: BufRead>(buf: &mut Vec<u8>, reader: &mut R) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;
    let mut fval = word.parse::<f32>().unwrap_or(0.0);
    fval /= 100.0;

    xsprint(buf, 's', XsprintValue::String(",f"));
    xsprint(buf, 'f', XsprintValue::Float(fval));
    Ok(buf.len())
}

/// Ported from `Xp_linf(char *buf, int k, float xmin, float lmaxmin, float xstep)`
pub fn x_p_linf<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    xmin: f32,
    lmaxmin: f32,
    mut xstep: f32,
) -> io::Result<usize> {
    let mut fval = x_r_float(reader)?;
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
    Ok(buf.len())
}

/// Ported from `Xp_logf(char *buf, int k, float xmin, float lmaxmin, int nsteps)`
pub fn x_p_logf<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    xmin: f32,
    lmaxmin: f32,
    nsteps: i32,
) -> io::Result<usize> {
    let mut fval = x_r_float(reader)?;

    // Safety check against invalid log input
    if fval <= 0.0 || xmin <= 0.0 {
         fval = 0.0;
    } else {
        fval = (fval / xmin).ln() / lmaxmin;
        let nsteps_f = nsteps as f32;
        fval = (fval * nsteps_f).round() / nsteps_f;
    }

    if fval <= 0.0 {
        fval = 0.0;
    }
    if fval > 1.0 {
        fval = 1.0;
    }

    xsprint(buf, 's', XsprintValue::String(",f"));
    xsprint(buf, 'f', XsprintValue::Float(fval));
    Ok(buf.len())
}

/// Ported from `Xp_int(char *buf, int k)`
pub fn x_p_int<R: BufRead>(buf: &mut Vec<u8>, reader: &mut R) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;
    let ival = word.parse::<i32>().unwrap_or(0);

    xsprint(buf, 's', XsprintValue::String(",i"));
    xsprint(buf, 'i', XsprintValue::Int(ival));
    Ok(buf.len())
}

/// Ported from `Xp_str(char *buf, int k)`
pub fn x_p_str<R: BufRead>(buf: &mut Vec<u8>, reader: &mut R) -> io::Result<usize> {
    // Read up to first '"'
    let mut scrap = Vec::new();
    reader.read_until(b'"', &mut scrap)?;

    // Read until second '"'
    let mut val_bytes = Vec::new();
    reader.read_until(b'"', &mut val_bytes)?;

    // Remove the trailing quote
    if val_bytes.last() == Some(&b'"') {
        val_bytes.pop();
    }

    let s = String::from_utf8_lossy(&val_bytes).into_owned();

    xsprint(buf, 's', XsprintValue::String(",s"));
    xsprint(buf, 's', XsprintValue::String(&s));
    Ok(buf.len())
}

/// Ported from `Xp_list(char *buf, int k, char **list, int list_max)`
pub fn x_p_list<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    list: &[&str],
) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    let mut ival = 0;
    for (i, &item) in list.iter().enumerate() {
        if word == item {
            ival = i as i32;
            break;
        }
    }

    xsprint(buf, 's', XsprintValue::String(",i"));
    xsprint(buf, 'i', XsprintValue::Int(ival));
    Ok(buf.len())
}

/// Ported from `Xp_fxlist(char *buf, int k, char **list, int list_max, int *p_ival)`
pub fn x_p_fxlist<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    list: &[&str],
) -> io::Result<(usize, i32)> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    let mut ival = 0;
    for (i, &item) in list.iter().enumerate() {
        if word == item {
            ival = i as i32;
            break;
        }
    }

    xsprint(buf, 's', XsprintValue::String(",i"));
    xsprint(buf, 'i', XsprintValue::Int(ival));
    Ok((buf.len(), ival))
}

/// Ported from `Xp_bit(char *buf, int k)`
pub fn x_p_bit<R: BufRead>(buf: &mut Vec<u8>, reader: &mut R) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    let mut ival = 0;
    if word.ends_with('%') {
        let bits_str = &word[..word.len() - 1];
        let mut j = 0;
        // Loop backwards
        for ch in bits_str.chars().rev() {
            if ch == '1' {
                ival |= 1 << j;
            }
            j += 1;
        }
    }

    xsprint(buf, 's', XsprintValue::String(",i"));
    xsprint(buf, 'i', XsprintValue::Int(ival));
    Ok(buf.len())
}

/// Ported from `Xp_frequency(char *buf, int k, int nsteps)`
pub fn x_p_frequency<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    nsteps: i32,
) -> io::Result<usize> {
    let mut fval = x_r_float(reader)?;

    if fval <= 0.0 {
        fval = 0.0;
    } else {
        fval = (fval / 20.0).ln() / 6.907755279;
        let nsteps_f = nsteps as f32;
        fval = (fval * nsteps_f).round() / nsteps_f;
    }

    if fval <= 0.0 {
        fval = 0.0;
    }
    if fval > 1.0 {
        fval = 1.0;
    }

    xsprint(buf, 's', XsprintValue::String(",f"));
    xsprint(buf, 'f', XsprintValue::Float(fval));
    Ok(buf.len())
}

/// Ported from `Xp_level(char *buf, int k, int nsteps)`
pub fn x_p_level<R: BufRead>(
    buf: &mut Vec<u8>,
    reader: &mut R,
    nsteps: i32,
) -> io::Result<usize> {
    let mut word = String::new();
    read_word(reader, &mut word)?;

    let mut fval;
    if word == "-oo" {
        fval = 0.0;
    } else {
        fval = word.parse::<f32>().unwrap_or(0.0);
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
    Ok(buf.len())
}


/// Helper function to read a word delimited by whitespace, similar to `fscanf(Xin, "%s", ...)`
fn read_word<R: BufRead>(reader: &mut R, word: &mut String) -> io::Result<()> {
    word.clear();
    let mut buf = [0; 1];
    let mut in_word = false;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let c = buf[0] as char;
        if c.is_whitespace() {
            if in_word {
                break;
            }
        } else {
            in_word = true;
            word.push(c);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_x_off_on() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("OFF");
        x_off_on(&mut buf, &mut cursor, "OFF").unwrap();

        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&0i32.to_be_bytes());
        assert_eq!(buf, expected);

        let mut buf = Vec::new();
        let mut cursor = Cursor::new("ON");
        x_off_on(&mut buf, &mut cursor, "OFF").unwrap();

        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_x_r_float() {
        let mut cursor = Cursor::new("3.14");
        assert_eq!(x_r_float(&mut cursor).unwrap(), 3.14);

        let mut cursor = Cursor::new("1k2");
        assert_eq!(x_r_float(&mut cursor).unwrap(), 1200.0);

        let mut cursor = Cursor::new("10k00");
        assert_eq!(x_r_float(&mut cursor).unwrap(), 10000.0);

        let mut cursor = Cursor::new("20");
        assert_eq!(x_r_float(&mut cursor).unwrap(), 20.0);
    }

    #[test]
    fn test_x_p_percent() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("50.0");
        x_p_percent(&mut buf, &mut cursor).unwrap();

        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&0.5f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_x_p_str() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("some ignore \"Hello World\" extra");
        x_p_str(&mut buf, &mut cursor).unwrap();

        let mut expected = b",s\0\0".to_vec();
        expected.extend_from_slice(b"Hello World\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_x_p_list() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("B");
        let list = vec!["A", "B", "C"];
        x_p_list(&mut buf, &mut cursor, &list).unwrap();

        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_x_p_bit() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("101%");
        x_p_bit(&mut buf, &mut cursor).unwrap();

        let mut expected = b",i\0\0".to_vec();
        expected.extend_from_slice(&5i32.to_be_bytes()); // 101 binary is 5
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_x_p_level() {
        let mut buf = Vec::new();
        let mut cursor = Cursor::new("-oo");
        x_p_level(&mut buf, &mut cursor, 1024).unwrap();

        let mut expected = b",f\0\0".to_vec();
        expected.extend_from_slice(&0.0f32.to_be_bytes());
        assert_eq!(buf, expected);
    }
}
