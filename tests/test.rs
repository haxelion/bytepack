extern crate bytepack;
#[macro_use]
extern crate bytepack_derive;

use std::io::Cursor;

use bytepack::{Packer, Unpacker, Packed};

#[test]
fn u64_exact() {
    let mut buffer = Cursor::new(vec![0u8, 64]);
    let case = [0u64, 1u64, 2u64, 3u64, 4u64, 5u64, 6u64, 7u64];
    let mut result = [0u64; 8];
    buffer.pack_all(&case).unwrap();
    buffer.set_position(0);
    buffer.unpack_exact(&mut result).unwrap();
    assert!(case == result);
}

#[test]
fn u64_to_end() {
    let mut buffer = Cursor::new(vec![0u8, 64]);
    let case = [0u64, 1u64, 2u64, 3u64, 4u64, 5u64, 6u64, 7u64];
    let mut result = Vec::<u64>::new();
    buffer.pack_all(&case).unwrap();
    buffer.set_position(0);
    buffer.unpack_to_end(&mut result).unwrap();
    assert!(case == &result[..]);
}

#[test]
fn f64_exact() {
    let mut buffer = Cursor::new(vec![0u8, 64]);
    let case = [0.0f64, 1.0f64, 2.0f64, 3.0f64, 4.0f64, 5.0f64, 6.0f64, 7.0f64];
    let mut result = [0f64; 8];
    buffer.pack_all(&case).unwrap();
    buffer.set_position(0);
    buffer.unpack_exact(&mut result).unwrap();
    assert!(case == result);
}

#[test]
fn f64_to_end() {
    let mut buffer = Cursor::new(vec![0u8, 64]);
    let case = [0.0f64, 1.0f64, 2.0f64, 3.0f64, 4.0f64, 5.0f64, 6.0f64, 7.0f64];
    let mut result = Vec::<f64>::new();
    buffer.pack_all(&case).unwrap();
    buffer.set_position(0);
    buffer.unpack_to_end(&mut result).unwrap();
    assert!(case == &result[..]);
}

#[test]
fn multiple() {
    let mut buffer = Cursor::new(vec![0u8, 128]);
    buffer.pack(1u8).unwrap();
    buffer.pack(-2i16).unwrap();
    buffer.pack(3u32).unwrap();
    buffer.pack(-4i64).unwrap();
    buffer.pack(5.0f32).unwrap();
    buffer.pack(-6.0f64).unwrap();
    buffer.set_position(0);
    assert!(buffer.unpack::<u8>().unwrap() == 1u8);
    assert!(buffer.unpack::<i16>().unwrap() == -2i16);
    assert!(buffer.unpack::<u32>().unwrap() == 3u32);
    assert!(buffer.unpack::<i64>().unwrap() == -4i64);
    assert!(buffer.unpack::<f32>().unwrap() == 5.0f32);
    assert!(buffer.unpack::<f64>().unwrap() == -6.0f64);
}

#[derive(Packed)]
struct Foo {
    a: u16,
    b: f32,
    c: i8
}

#[test]
fn struct_unpack() {
    let mut buffer = Cursor::new(vec![0u8, 128]);
    buffer.pack(Foo {a: 666u16, b: 3.14f32, c: -42i8}).unwrap();
    buffer.set_position(0);
    let foo : Foo = buffer.unpack().unwrap();
    assert!(foo.a == 666u16);
    assert!(foo.b == 3.14f32);
    assert!(foo.c == -42i8);
}
