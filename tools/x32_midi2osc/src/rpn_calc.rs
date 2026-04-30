#![allow(dead_code)]
use anyhow::{Result, anyhow};

pub struct RpnCalculator {
    memory: f64,
}

impl RpnCalculator {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }

    pub fn calculate(&mut self, expr: &str, mparam: &[f64; 3]) -> Result<f64> {
        let mut stack: Vec<f64> = Vec::new();

        let tokens = expr.split_whitespace();

        for token in tokens {
            if token.starts_with('$') {
                if let Ok(idx) = token[1..].parse::<usize>() {
                    if idx < mparam.len() {
                        stack.push(mparam[idx]);
                    } else {
                        return Err(anyhow!("Parameter index out of bounds: {}", token));
                    }
                } else {
                    return Err(anyhow!("Invalid parameter format: {}", token));
                }
            } else if let Ok(num) = token.parse::<f64>() {
                stack.push(num);
            } else {
                match token {
                    "?" => {
                        let c = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ?"))?;
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ?"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ?"))?;
                        stack.push(if a != 0.0 { b } else { c });
                    }
                    "+" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on +"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on +"))?;
                        stack.push(a + b);
                    }
                    "-" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on -"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on -"))?;
                        stack.push(a - b);
                    }
                    "*" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on *"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on *"))?;
                        stack.push(a * b);
                    }
                    "/" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on /"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on /"))?;
                        if b == 0.0 {
                            return Err(anyhow!("Division by zero"));
                        }
                        stack.push(a / b);
                    }
                    "e" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on e"))?;
                        stack.push(a.exp());
                    }
                    "l" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on l"))?;
                        stack.push(a.ln());
                    }
                    "L" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on L"))?;
                        stack.push(a.log10());
                    }
                    "%" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on %"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on %"))?;
                        stack.push((a as i64 % b as i64) as f64);
                    }
                    "^" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ^"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ^"))?;
                        stack.push((a as i64 ^ b as i64) as f64);
                    }
                    "&" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on &"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on &"))?;
                        stack.push((a as i64 & b as i64) as f64);
                    }
                    "|" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on |"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on |"))?;
                        stack.push((a as i64 | b as i64) as f64);
                    }
                    ">" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on >"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on >"))?;
                        stack.push(if a > b { 1.0 } else { 0.0 });
                    }
                    "<" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on <"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on <"))?;
                        stack.push(if a < b { 1.0 } else { 0.0 });
                    }
                    "=" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ="))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ="))?;
                        stack.push(if (a as i64) == (b as i64) { 1.0 } else { 0.0 });
                    }
                    "!" => {
                        let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow on !"))?;
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on !"))?;
                        stack.push(if (a as i64) != (b as i64) { 1.0 } else { 0.0 });
                    }
                    "~" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on ~"))?;
                        stack.push((!(a as i64)) as f64);
                    }
                    "i" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on i"))?;
                        stack.push(a.trunc());
                    }
                    "a" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on a"))?;
                        stack.push(a.abs());
                    }
                    "M" => {
                        let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow on M"))?;
                        self.memory += a;
                    }
                    "m" => {
                        stack.push(self.memory);
                    }
                    "Z" => {
                        self.memory = 0.0;
                    }
                    _ => {
                        return Err(anyhow!("Unknown operator: {}", token));
                    }
                }
            }
        }

        if stack.len() == 1 {
            Ok(stack[0])
        } else if stack.is_empty() {
            // Some operations like 'M' or 'Z' leave stack empty, this might be expected if the user just wants side effects,
            // but the C code returns `pop()` at the end which errors on underflow. We'll return 0.0 on empty if that happens.
            Ok(0.0)
        } else {
            Err(anyhow!("Stack leftover items: {:?}", stack))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut calc = RpnCalculator::new();
        let mparam = [0.0, 0.0, 0.0];
        assert_eq!(calc.calculate("3 4 +", &mparam).unwrap(), 7.0);
        assert_eq!(calc.calculate("10 2 -", &mparam).unwrap(), 8.0);
        assert_eq!(calc.calculate("3 4 *", &mparam).unwrap(), 12.0);
        assert_eq!(calc.calculate("12 3 /", &mparam).unwrap(), 4.0);
    }

    #[test]
    fn test_parameters() {
        let mut calc = RpnCalculator::new();
        let mparam = [1.0, 7.0, 127.0];
        assert_eq!(calc.calculate("$2 127 /", &mparam).unwrap(), 1.0);
        assert_eq!(calc.calculate("$1 3 +", &mparam).unwrap(), 10.0);
        assert_eq!(calc.calculate("$0", &mparam).unwrap(), 1.0);
    }

    #[test]
    fn test_memory() {
        let mut calc = RpnCalculator::new();
        let mparam = [0.0, 0.0, 0.0];
        calc.calculate("5 M", &mparam).unwrap(); // memory = 5
        assert_eq!(calc.calculate("m 2 +", &mparam).unwrap(), 7.0);
        calc.calculate("Z", &mparam).unwrap(); // memory = 0
        assert_eq!(calc.calculate("m", &mparam).unwrap(), 0.0);
    }

    #[test]
    fn test_bitwise() {
        let mut calc = RpnCalculator::new();
        let mparam = [0.0, 0.0, 0.0];
        assert_eq!(calc.calculate("5 3 &", &mparam).unwrap(), 1.0);
        assert_eq!(calc.calculate("5 3 |", &mparam).unwrap(), 7.0);
        assert_eq!(calc.calculate("5 3 ^", &mparam).unwrap(), 6.0);
    }
}
