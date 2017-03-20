bytepack
========

[![Crates.io](https://img.shields.io/crates/v/bytepack.svg)](https://crates.io/crates/bytepack)
[![Build Status](https://travis-ci.org/haxelion/bytepack.svg?branch=master)](https://travis-ci.org/haxelion/bytepack)
[![Docs.rs](https://docs.rs/bytepack/badge.svg)](https://docs.rs/bytepack)

`bytepack` is a simple crate which extends the `std::io` API to be able to read and write any 
data type in their memory representation. It can be seen as a generalization of the 
`std::io::Read` and `std::io::Write` trait , but operating on a generic parameter `T` instead 
of `u8`. This crate focus on performances by beeing no copy (except in one clearly marked case) 
and offering methods to read and write arrays.

`bytepack` offers three trait famillies allowing different endianness control. 
`Unpacker` and `Packer` read and write data in the endianness of the operating system. `LEUnpacker` 
and `LEPacker` always read and write data in little endian while `BEUnpacker` and `BEPacker` do the 
same in big endian. They all conform to the same API which is copied from the one of `std::io`.
This means switching from one endianness to another can be done by simply bringing a different 
trait in scope.

Because `bytepack` is not a serialization library, it cannot read and write complex types like 
`Vec`, `Rc`, etc. directly from a Reader or to Writer. Indeed those types do not contain the 
underlying data directly packed inside but rather hold a reference or a pointer to it. To 
identify types which holds their data "packed" together, the `Packed` trait is used. Additionnaly 
it provides a in-place endianness switching method. One can implement this trait for the data types 
deemed safe to read and write. An automatic derive for structures made only of types implementing 
`Packed` is also implemented in the [`bytepack_derive`](https://github.com/haxelion/bytepack_derive) 
crate.

Example
-------

Here are two functions which can serialize and deserialize a `Vec<f32>`:

``` rust
extern crate bytepack;

use std::fs::File;
use std::iter::repeat;

use bytepack::{LEPacker, LEUnpacker};

fn write_samples(file: &str, samples: &Vec<f32>) {
    let mut file = File::create(file).unwrap();
    file.pack(samples.len() as u32).unwrap();
    file.pack_all(&samples[..]).unwrap();
}

fn read_samples(file: &str) -> Vec<f32> {
    let mut file = File::open(file).unwrap();
    let num_samples : u32 = file.unpack().unwrap();
    let mut samples : Vec<f32> = repeat(0f32).take(num_samples as usize).collect();
    file.unpack_exact(&mut samples[..]).unwrap();
    return samples;
}
```

Thanks to the genericity of the `Packed` trait we could make the previous function generic:

``` rust
extern crate bytepack;

use std::fs::File;

use bytepack::{LEPacker, Packed};

fn write_vec<T: Packed + Clone>(file: &str, samples: &Vec<T>) {
    let mut file = File::create(file).unwrap();
    file.pack(samples.len() as u32).unwrap();
    file.pack_all(&samples[..]).unwrap();
}
```

Because we can derive Packed for our own struct, we can use them with our own types:

``` rust
extern crate bytepack;
#[macro_use]
extern crate bytepack_derive;

use std::fs::File;

use bytepack::{LEPacker, Packed};

#[derive(Packed, Clone)]
struct Vertex<T: Packed> {
    x: T,
    y: T,
    z: T
}

fn write_vec<T: Packed + Clone>(file: &str, samples: &Vec<T>) {
    let mut file = File::create(file).unwrap();
    file.pack(samples.len() as u32).unwrap();
    file.pack_all(&samples[..]).unwrap();
}

fn main() {
    let square : Vec<Vertex<f32>> = vec![
        Vertex{x: 0.0, y: 0.0, z: 0.0},
        Vertex{x: 1.0, y: 0.0, z: 0.0},
        Vertex{x: 1.0, y: 1.0, z: 0.0},
        Vertex{x: 0.0, y: 1.0, z: 0.0},
    ];

    write_vec("square.raw", &square);
}
```


