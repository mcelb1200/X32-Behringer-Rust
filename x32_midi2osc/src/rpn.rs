use anyhow::{Result, anyhow};

pub struct RpnCalculator {
    memory: f64,
}

impl Default for RpnCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl RpnCalculator {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }

    /// Evaluates a space-separated RPN expression.
    /// The expression can use `$1`, `$2`, etc. to refer to values in `mparam` (1-based or 0-based depending on usage,
    /// in C it was `strtol(++s)`, so `$0` is mparam[0], `$1` is mparam[1], etc.).
    /// `mparam` array is expected to hold at least enough parameters.
    pub fn evaluate(&mut self, expr: &str, mparam: &[f64]) -> Result<f64> {
        let mut stack: Vec<f64> = Vec::new();

        let tokens = expr.split_whitespace();

        for token in tokens {
            if let Some(stripped) = token.strip_prefix('$') {
                let idx: usize = stripped.parse().unwrap_or(0);
                let val = if idx < mparam.len() { mparam[idx] } else { 0.0 };
                stack.push(val);
            } else if let Ok(val) = token.parse::<f64>() {
                stack.push(val);
            } else {
                // Parse operator
                match token {
                    "?" => {
                        let c = stack.pop().unwrap_or(0.0);
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if a != 0.0 { b } else { c });
                    }
                    "+" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a + b);
                    }
                    "-" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a - b);
                    }
                    "*" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a * b);
                    }
                    "/" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if b != 0.0 { a / b } else { 0.0 });
                    }
                    "e" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a.exp());
                    }
                    "l" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a.ln());
                    }
                    "L" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a.log10());
                    }
                    "%" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        let bb = b as i32;
                        stack.push(if bb != 0 {
                            ((a as i32) % bb) as f64
                        } else {
                            0.0
                        });
                    }
                    "^" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(((a as i32) ^ (b as i32)) as f64);
                    }
                    "&" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(((a as i32) & (b as i32)) as f64);
                    }
                    "|" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(((a as i32) | (b as i32)) as f64);
                    }
                    ">" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if a > b { 1.0 } else { 0.0 });
                    }
                    "<" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if a < b { 1.0 } else { 0.0 });
                    }
                    "=" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if (a as i32) == (b as i32) { 1.0 } else { 0.0 });
                    }
                    "!" => {
                        let b = stack.pop().unwrap_or(0.0);
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(if (a as i32) != (b as i32) { 1.0 } else { 0.0 });
                    }
                    "~" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push((!(a as i32)) as f64);
                    }
                    "i" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push((a as i32) as f64);
                    }
                    "a" => {
                        let a = stack.pop().unwrap_or(0.0);
                        stack.push(a.abs());
                    }
                    "M" => {
                        self.memory += stack.pop().unwrap_or(0.0);
                    }
                    "m" => {
                        stack.push(self.memory);
                    }
                    "Z" => {
                        self.memory = 0.0;
                    }
                    _ => {
                        return Err(anyhow!("Unknown operator or invalid number: {}", token));
                    }
                }
            }
        }

        if stack.len() > 1 {
            return Err(anyhow!("Leftover values on stack after evaluation"));
        }

        Ok(stack.pop().unwrap_or(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        let mut calc = RpnCalculator::new();
        assert_eq!(calc.evaluate("2 3 +", &[]).unwrap(), 5.0);
        assert_eq!(calc.evaluate("5 2 -", &[]).unwrap(), 3.0);
        assert_eq!(calc.evaluate("4 3 *", &[]).unwrap(), 12.0);
        assert_eq!(calc.evaluate("10 2 /", &[]).unwrap(), 5.0);
    }

    #[test]
    fn test_variables() {
        let mut calc = RpnCalculator::new();
        let params = [10.0, 20.0, 30.0];
        assert_eq!(calc.evaluate("$0 $1 +", &params).unwrap(), 30.0);
        assert_eq!(calc.evaluate("$2 $0 -", &params).unwrap(), 20.0);
        assert_eq!(calc.evaluate("$3", &params).unwrap(), 0.0); // Out of bounds returns 0
    }

    #[test]
    fn test_bitwise_logic() {
        let mut calc = RpnCalculator::new();
        // 5 & 3 = 1
        assert_eq!(calc.evaluate("5 3 &", &[]).unwrap(), 1.0);
        // 5 | 2 = 7
        assert_eq!(calc.evaluate("5 2 |", &[]).unwrap(), 7.0);
        // 5 ^ 3 = 6
        assert_eq!(calc.evaluate("5 3 ^", &[]).unwrap(), 6.0);
        // ~1
        assert_eq!(calc.evaluate("1 ~", &[]).unwrap(), (!1) as f64);
    }

    #[test]
    fn test_comparisons() {
        let mut calc = RpnCalculator::new();
        assert_eq!(calc.evaluate("5 5 =", &[]).unwrap(), 1.0);
        assert_eq!(calc.evaluate("5 6 =", &[]).unwrap(), 0.0);
        assert_eq!(calc.evaluate("5 6 !", &[]).unwrap(), 1.0);
        assert_eq!(calc.evaluate("5 6 <", &[]).unwrap(), 1.0);
        assert_eq!(calc.evaluate("6 5 <", &[]).unwrap(), 0.0);
        assert_eq!(calc.evaluate("6 5 >", &[]).unwrap(), 1.0);
        assert_eq!(calc.evaluate("5 6 >", &[]).unwrap(), 0.0);
    }

    #[test]
    fn test_ternary() {
        let mut calc = RpnCalculator::new();
        // a b c ? -> if a { b } else { c }
        // 1 10 20 ? -> 10
        assert_eq!(calc.evaluate("1 10 20 ?", &[]).unwrap(), 10.0);
        // 0 10 20 ? -> 20
        assert_eq!(calc.evaluate("0 10 20 ?", &[]).unwrap(), 20.0);
    }

    #[test]
    fn test_memory() {
        let mut calc = RpnCalculator::new();
        calc.evaluate("10 M", &[]).unwrap();
        assert_eq!(calc.memory, 10.0);
        calc.evaluate("5 M", &[]).unwrap();
        assert_eq!(calc.memory, 15.0);
        assert_eq!(calc.evaluate("m 2 +", &[]).unwrap(), 17.0);
        calc.evaluate("Z", &[]).unwrap();
        assert_eq!(calc.memory, 0.0);
    }

    #[test]
    fn test_math_functions() {
        let mut calc = RpnCalculator::new();
        // abs
        assert_eq!(calc.evaluate("-5 a", &[]).unwrap(), 5.0);
        // int cast
        assert_eq!(calc.evaluate("5.7 i", &[]).unwrap(), 5.0);
    }

    #[test]
    fn test_m2o_example() {
        // e.g. [$2 127.0 /]
        let mut calc = RpnCalculator::new();
        let params = [0.0, 0.0, 63.5];
        assert_eq!(calc.evaluate("$2 127.0 /", &params).unwrap(), 0.5);
    }
}
