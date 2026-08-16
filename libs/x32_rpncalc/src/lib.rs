//! A Reverse Polish Notation (RPN) calculator used for evaluating complex
//! parameter dependencies, porting `X32RpnCalc.c` from the original C codebase.

/// Represents a value that can be passed as a parameter (`$n`) in the expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamValue {
    /// An integer parameter
    Int(i32),
    /// A floating-point parameter
    Float(f64),
}

impl ParamValue {
    /// Returns the value as an `f64`.
    pub fn as_f64(&self) -> f64 {
        match self {
            ParamValue::Int(i) => *i as f64,
            ParamValue::Float(f) => *f,
        }
    }
}

/// An RPN (Reverse Polish Notation) calculator.
#[derive(Debug, Default)]
pub struct RpnCalc {
    /// The evaluation stack
    stack: Vec<f64>,
    /// The single-slot memory
    memory: f64,
}

impl RpnCalc {
    /// Creates a new `RpnCalc` instance.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            memory: 0.0,
        }
    }

    /// Evaluates an RPN expression string.
    ///
    /// The `params` parameter allows passing values that can be referenced using `$n` in the expression.
    ///
    /// Returns a tuple containing:
    /// - The evaluated result (`f64`)
    /// - The number of bytes consumed from the input string
    ///
    /// Returns an error message (`String`) on stack overflow/underflow or invalid input.
    pub fn evaluate(
        &mut self,
        expression: &str,
        params: Option<&[ParamValue]>,
    ) -> Result<(f64, usize), String> {
        let mut bytes_consumed = 0;
        let bytes = expression.as_bytes();
        let mut i = 0;

        self.stack.clear();

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Stop on `]`
            if ch == ']' {
                bytes_consumed = i + 1;
                break;
            }

            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            // Parameter reference: `$n`
            if ch == '$' {
                i += 1;
                let mut param_idx_str = String::new();
                while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                    param_idx_str.push(bytes[i] as char);
                    i += 1;
                }

                let val = if let Ok(idx) = param_idx_str.parse::<usize>() {
                    if let Some(p) = params {
                        p.get(idx).map(|pv| pv.as_f64()).unwrap_or(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                self.push(val)?;
                continue;
            }

            // Numeric literal
            // It could start with a digit, a decimal point, or a sign (+/-) followed by digit/point.
            // But beware that '+' and '-' are also operators if standalone.
            let mut is_number = false;
            let mut end_idx = i;

            // Check if this could be a number literal.
            // Be careful not to eagerly parse `-` or `+` if they don't precede a digit/dot.
            if ch.is_ascii_digit() || ch == '.' {
                is_number = true;
            } else if (ch == '+' || ch == '-') && i + 1 < bytes.len() {
                let next_ch = bytes[i + 1] as char;
                if next_ch.is_ascii_digit() || next_ch == '.' {
                    is_number = true;
                }
            }

            if is_number {
                // If it starts with + or -, advance end_idx past it initially
                // so we don't trip over it in the while loop below
                if ch == '+' || ch == '-' {
                    end_idx += 1;
                }

                while end_idx < bytes.len() {
                    let c = bytes[end_idx] as char;
                    if c.is_ascii_digit()
                        || c == '.'
                        || c == 'e'
                        || c == 'E'
                        || (end_idx > 0
                            && c == '+'
                            && (bytes[end_idx - 1] as char == 'e'
                                || bytes[end_idx - 1] as char == 'E'))
                        || (end_idx > 0
                            && c == '-'
                            && (bytes[end_idx - 1] as char == 'e'
                                || bytes[end_idx - 1] as char == 'E'))
                    {
                        end_idx += 1;
                    } else {
                        break;
                    }
                }

                let num_str = &expression[i..end_idx];
                if let Ok(val) = num_str.parse::<f64>() {
                    self.push(val)?;
                    i = end_idx;
                    continue;
                } else {
                    // It's possible that a '-' or '+' is parsed as an operator, not part of a number literal,
                    // if parsing failed (e.g., "-"). But we already verified next char is digit/., so failure here is unusual.
                    return Err(format!("Invalid number literal: '{}'", num_str));
                }
            }

            // Operators
            match ch {
                '?' => {
                    let c = self.pop()?;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if a != 0.0 { b } else { c })?;
                }
                '+' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a + b)?;
                }
                '-' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a - b)?;
                }
                '*' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a * b)?;
                }
                '/' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a / b)?;
                }
                'e' => {
                    let a = self.pop()?;
                    self.push(a.exp())?;
                }
                'l' => {
                    let a = self.pop()?;
                    self.push(a.ln())?; // natural log
                }
                'L' => {
                    let a = self.pop()?;
                    self.push(a.log10())?;
                }
                '%' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let a_i = a as i32;
                    let b_i = b as i32;
                    if let Some(res) = a_i.checked_rem(b_i) {
                        self.push(res as f64)?;
                    } else {
                        // Return 0.0 or Err. In this case, 0.0 is a safe fallback to match div by zero usually yielding something
                        self.push(0.0)?;
                    }
                }
                '^' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(((a as i32) ^ (b as i32)) as f64)?;
                }
                '&' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(((a as i32) & (b as i32)) as f64)?;
                }
                '|' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(((a as i32) | (b as i32)) as f64)?;
                }
                '>' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(((a as i32).wrapping_shr(b as u32)) as f64)?;
                }
                '<' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(((a as i32).wrapping_shl(b as u32)) as f64)?;
                }
                '=' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if (a as i32) == (b as i32) { 1.0 } else { 0.0 })?;
                }
                '!' => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(if (a as i32) != (b as i32) { 1.0 } else { 0.0 })?;
                }
                '~' => {
                    let a = self.pop()?;
                    self.push((!(a as i32)) as f64)?;
                }
                'i' => {
                    let a = self.pop()?;
                    self.push((a as i32) as f64)?;
                }
                'a' => {
                    let a = self.pop()?;
                    self.push(a.abs())?;
                }
                'M' => {
                    let val = self.pop()?;
                    self.memory += val;
                }
                'm' => {
                    self.push(self.memory)?;
                }
                'Z' => {
                    self.memory = 0.0;
                }
                _ => {
                    // Ignore unknown characters? C code has `noop()` for unknown.
                    // Let's just ignore to match `noop()`.
                }
            }

            i += 1;
        }

        if bytes_consumed == 0 {
            bytes_consumed = i; // if it didn't end with `]`
        }

        if self.stack.len() > 1 {
            return Err("stack leftover".to_string());
        }

        let result = if self.stack.is_empty() {
            0.0
        } else {
            self.pop()?
        };

        Ok((result, bytes_consumed))
    }

    fn push(&mut self, val: f64) -> Result<(), String> {
        if self.stack.len() >= 128 {
            return Err("stack overflow".to_string());
        }
        self.stack.push(val);
        Ok(())
    }

    fn pop(&mut self) -> Result<f64, String> {
        self.stack
            .pop()
            .ok_or_else(|| "stack underflow".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        let mut calc = RpnCalc::new();
        // 3 4 + = 7
        assert_eq!(calc.evaluate("3 4 +", None).unwrap().0, 7.0);

        // 10 2 / = 5
        assert_eq!(calc.evaluate("10 2 /", None).unwrap().0, 5.0);

        // 2 3 * 4 - = 2
        assert_eq!(calc.evaluate("2 3 * 4 -", None).unwrap().0, 2.0);
    }

    #[test]
    fn test_parameters() {
        let mut calc = RpnCalc::new();
        let params = vec![ParamValue::Int(10), ParamValue::Float(2.5)];

        // $0 $1 * = 10 * 2.5 = 25
        assert_eq!(calc.evaluate("$0 $1 *", Some(&params)).unwrap().0, 25.0);
    }

    #[test]
    fn test_memory() {
        let mut calc = RpnCalc::new();
        // 5 M -> adds 5 to memory, leaves stack empty
        // m -> pushes memory (5)
        // 2 * -> 5 * 2 = 10
        assert_eq!(calc.evaluate("5 M m 2 *", None).unwrap().0, 10.0);

        // check memory retained
        // m -> pushes 5
        assert_eq!(calc.evaluate("m", None).unwrap().0, 5.0);

        // Z -> zeros memory
        // m -> pushes 0
        assert_eq!(calc.evaluate("Z m", None).unwrap().0, 0.0);
    }

    #[test]
    fn test_logic_and_bitwise() {
        let mut calc = RpnCalc::new();

        // 5 3 % -> 2

        // 1 0 = -> 0
        assert_eq!(calc.evaluate("1 0 =", None).unwrap().0, 0.0);

        // 1 1 = -> 1
        assert_eq!(calc.evaluate("1 1 =", None).unwrap().0, 1.0);

        // 1 0 ! -> 1
        assert_eq!(calc.evaluate("1 0 !", None).unwrap().0, 1.0);

        // Bitwise logic
        // 5 3 & -> 1
        assert_eq!(calc.evaluate("5 3 &", None).unwrap().0, 1.0);

        // 5 3 | -> 7
        assert_eq!(calc.evaluate("5 3 |", None).unwrap().0, 7.0);

        // 5 3 ^ -> 6
        assert_eq!(calc.evaluate("5 3 ^", None).unwrap().0, 6.0);

        // Bit shifts
        // 1 3 < -> 8 (1 << 3)
        assert_eq!(calc.evaluate("1 3 <", None).unwrap().0, 8.0);

        // 8 2 > -> 2 (8 >> 2)
        assert_eq!(calc.evaluate("8 2 >", None).unwrap().0, 2.0);
    }

    #[test]
    fn test_ternary_conditional() {
        let mut calc = RpnCalc::new();
        // a b c ? => if a != 0 then b else c

        // 1 10 20 ? -> 10
        assert_eq!(calc.evaluate("1 10 20 ?", None).unwrap().0, 10.0);

        // 0 10 20 ? -> 20
        assert_eq!(calc.evaluate("0 10 20 ?", None).unwrap().0, 20.0);
    }

    #[test]
    fn test_unary_ops() {
        let mut calc = RpnCalc::new();

        // abs
        assert_eq!(calc.evaluate("-5 a", None).unwrap().0, 5.0);

        // int cast
        assert_eq!(calc.evaluate("3.14 i", None).unwrap().0, 3.0);
    }

    #[test]
    fn test_termination() {
        let mut calc = RpnCalc::new();
        // 2 3 + ] 4 5 + => evaluates 2 3 + and stops, returning 5 and consumed up to ]
        let expr = "2 3 + ] 4 5 +";
        let (res, consumed) = calc.evaluate(expr, None).unwrap();
        assert_eq!(res, 5.0);
        assert_eq!(consumed, 7); // "2 3 + ]" length
    }
}

#[test]
fn test_modulo_by_zero_does_not_panic() {
    let mut calc = RpnCalc::new();
    // 5 0 % -> shouldn't panic
    assert_eq!(calc.evaluate("5 0 %", None).unwrap().0, 0.0);
}
