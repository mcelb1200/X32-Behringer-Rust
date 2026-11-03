struct A {
    path: String,
    args: Vec<i32>,
}
fn foo(path: &str, args: Vec<i32>) {}

fn main() {
    let a = A {
        path: String::new(),
        args: vec![],
    };
    foo(&a.path, a.args);
}
