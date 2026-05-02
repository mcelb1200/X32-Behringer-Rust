fn main() {
    let line = "/ch/01/eq 1 20.0 1.0";
    let (path, arg_str) = line.split_once(|c: char| c.is_whitespace()).unwrap();
    println!("path: {}, arg_str: {}", path, arg_str);
}
