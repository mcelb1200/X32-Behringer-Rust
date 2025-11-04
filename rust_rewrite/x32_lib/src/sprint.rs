
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

pub fn xsprint_s(buf: &mut Vec<u8>, s: &str) {
    buf.write_all(s.as_bytes()).unwrap();
    buf.write_all(&[0]).unwrap();
    while buf.len() % 4 != 0 {
        buf.write_all(&[0]).unwrap();
    }
}

pub fn xsprint_i(buf: &mut Vec<u8>, i: i32) {
    buf.write_i32::<BigEndian>(i).unwrap();
}

pub fn xsprint_f(buf: &mut Vec<u8>, f: f32) {
    buf.write_f32::<BigEndian>(f).unwrap();
}

pub fn xfprint_s(buf: &mut Vec<u8>, text: &str, s: &str) {
    xsprint_s(buf, text);
    xsprint_s(buf, ",s");
    xsprint_s(buf, s);
}

pub fn xfprint_i(buf: &mut Vec<u8>, text: &str, i: i32) {
    xsprint_s(buf, text);
    xsprint_s(buf, ",i");
    xsprint_i(buf, i);
}

pub fn xfprint_f(buf: &mut Vec<u8>, text: &str, f: f32) {
    xsprint_s(buf, text);
    xsprint_s(buf, ",f");
    xsprint_f(buf, f);
}
