//! `bytepack` is a simple crate which extends the `std::io` API to be able to read and write any 
//! data type in their memory representation. It can be seen as a generalization of the 
//! `std::io::Read` and `std::io::Write` trait , but operating on a generic parameter `T` instead 
//! of `u8`. This crate focus on performances by beeing no copy (except in one clearly marked case) 
//! and offering methods to read and write arrays.
//! 
//! `bytepack` offers three trait famillies allowing different endianness control. 
//! [`Unpacker`](trait.Unpacked.html) and [`Packer`](trait.Packer.html) read and write data in the 
//! endianness of the operating system. [`LEUnpacker`](trait.LEUnpacker.html) and 
//! [`LEPacker`](trait.LEPacker.html) always read and write data in little endian while 
//! [`BEUnpacker`](trait.BEUnpacker.html) and [`BEPacker`](trait.BEPacker.html) do the 
//! same in big endian. They all conform to the same API which is copied from the one of `std::io`.
//! This means switching from one endianness to another can be done by simply bringing a different 
//! trait in scope.
//!
//! Because `bytepack` is not a serialization library, it cannot read and write complex types like 
//! `Vec`, `Rc`, etc. directly from a Reader or to Writer. Indeed those types do not contain the 
//! underlying data directly packed inside but rather hold a reference or a pointer to it. To 
//! identify types which holds their data "packed" together, the [`Packed`](trait.Packed.html) 
//! trait is used. Additionnaly it provides a in-place endianness switching method. One can 
//! implement this trait for the data types deemed safe to read and write. A custom derive for 
//! structures made only of types implementing [`Packed`](trait.Packed.html) also exists.
//!
//! # Example
//!
//! ```no_run
//! use std::fs::File;
//! use std::iter::repeat;
//!
//! use bytepack::{LEPacker, LEUnpacker};
//!
//! fn write_samples(file: &str, samples: &Vec<f32>) {
//!     let mut file = File::create(file).unwrap();
//!     file.pack(samples.len() as u32).unwrap();
//!     file.pack_all(&samples[..]).unwrap();
//! }
//!
//! fn read_samples(file: &str) -> Vec<f32> {
//!     let mut file = File::open(file).unwrap();
//!     let num_samples : u32 = file.unpack().unwrap();
//!     let mut samples : Vec<f32> = repeat(0f32).take(num_samples as usize).collect();
//!     file.unpack_exact(&mut samples[..]).unwrap();
//!     return samples;
//! }
//! ```

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::mem::{zeroed, transmute, size_of, forget};
use std::slice;

/// This trait both identifies a type which holds his data packed together in memory and a type 
/// which offers a `switch_endianness` method. This trait is voluntarily not implemented for 
/// `isize` and `usize` because their size can vary from one system to another.
///
/// # Example
///
/// If you would like to read and write one of your struct using `bytepack`, you can derive 
/// `Packed` for it:
///
/// ```no_run
/// extern crate bytepack;
/// #[macro_use]
/// extern crate bytepack_derive;
/// 
/// use std::fs::File;
/// use bytepack::{LEUnpacker, Packed};
///
/// #[derive(Packed)]
/// struct Vector {
///    x: f32,
///    y: f32,
///    z: f32,
/// }
///
/// #[derive(Packed)]
/// struct RGB(u8,u8,u8);
///
/// fn main() {
///     let mut file = File::open("test").unwrap();
///     let vector : Vector = file.unpack().unwrap();
///     let rgb : RGB = file.unpack().unwrap();
/// }
///
/// ```
///
/// Please note that also specifying `#[repr(packed)]` might make sense if you want to get rid of 
/// the padding inside your structure.
/// 
/// `Packed` can only be derived for strutures only composed of types implementing `Packed` 
/// themselves. If you which to circumvent this restriction you can implement `Packed` yourselve, 
/// however you need to make sure your struct is indeed "packed" and that reading and writing it as 
/// one continuous memory zone makes sense. For example the following structures are not "packed" 
/// because they all hold a reference to their data.
///
/// ```ignore
/// struct NotPacked1 {
///     name: String
/// }
///
/// struct NotPacked2 {
///     numbers: Vec<f32>
/// }
///
/// struct NotPacked3 {
///     count: Rc<u64>
/// }
/// ```
pub trait Packed {
    /// Perform an in-place switch of the endianness. This might be a no-op in some cases.
    fn switch_endianness(&mut self);
}

impl Packed for bool {
    fn switch_endianness(&mut self) {
    }
}

impl Packed for u8 {
    fn switch_endianness(&mut self) {
    }
}

impl Packed for i8 {
    fn switch_endianness(&mut self) {
    }
}

impl Packed for u16 {
    fn switch_endianness(&mut self) {
        *self = u16::swap_bytes(*self);
    }
}

impl Packed for i16 {
    fn switch_endianness(&mut self) {
        *self = i16::swap_bytes(*self);
    }
}

impl Packed for u32 {
    fn switch_endianness(&mut self) {
        *self = u32::swap_bytes(*self);
    }
}

impl Packed for i32 {
    fn switch_endianness(&mut self) {
        *self = i32::swap_bytes(*self);
    }
}

impl Packed for u64 {
    fn switch_endianness(&mut self) {
        *self = u64::swap_bytes(*self);
    }
}

impl Packed for i64 {
    fn switch_endianness(&mut self) {
        *self = i64::swap_bytes(*self);
    }
}

impl Packed for f32 {
    fn switch_endianness(&mut self) {
        // Safe because we always revert to the original type
        unsafe {
            *self = transmute(u32::swap_bytes(transmute(*self)));
        }
    }
}

impl Packed for f64 {
    fn switch_endianness(&mut self) {
        // Safe because we always revert to the original type
        unsafe {
            *self = transmute(u64::swap_bytes(transmute(*self)));
        }
    }
}

impl<T> Packed for [T;1] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
    }
}

impl<T> Packed for [T;2] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
    }
}

impl<T> Packed for [T;3] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
    }
}

impl<T> Packed for [T;4] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
    }
}

impl<T> Packed for [T;5] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
    }
}

impl<T> Packed for [T;6] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
    }
}

impl<T> Packed for [T;7] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
    }
}

impl<T> Packed for [T;8] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
    }
}

impl<T> Packed for [T;9] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
    }
}

impl<T> Packed for [T;10] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
    }
}

impl<T> Packed for [T;11] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
    }
}

impl<T> Packed for [T;12] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
    }
}

impl<T> Packed for [T;13] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
    }
}

impl<T> Packed for [T;14] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
    }
}

impl<T> Packed for [T;15] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
    }
}

impl<T> Packed for [T;16] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
    }
}

impl<T> Packed for [T;17] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
    }
}

impl<T> Packed for [T;18] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
    }
}

impl<T> Packed for [T;19] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
    }
}

impl<T> Packed for [T;20] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
    }
}

impl<T> Packed for [T;21] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
    }
}

impl<T> Packed for [T;22] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
    }
}

impl<T> Packed for [T;23] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
    }
}

impl<T> Packed for [T;24] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
    }
}

impl<T> Packed for [T;25] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
    }
}

impl<T> Packed for [T;26] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
    }
}

impl<T> Packed for [T;27] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
    }
}

impl<T> Packed for [T;28] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
        self[27].switch_endianness();
    }
}

impl<T> Packed for [T;29] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
        self[27].switch_endianness();
        self[28].switch_endianness();
    }
}

impl<T> Packed for [T;30] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
        self[27].switch_endianness();
        self[28].switch_endianness();
        self[29].switch_endianness();
    }
}

impl<T> Packed for [T;31] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
        self[27].switch_endianness();
        self[28].switch_endianness();
        self[29].switch_endianness();
        self[30].switch_endianness();
    }
}

impl<T> Packed for [T;32] where T: Packed {
    fn switch_endianness(&mut self) {
        self[0].switch_endianness();
        self[1].switch_endianness();
        self[2].switch_endianness();
        self[3].switch_endianness();
        self[4].switch_endianness();
        self[5].switch_endianness();
        self[6].switch_endianness();
        self[7].switch_endianness();
        self[8].switch_endianness();
        self[9].switch_endianness();
        self[10].switch_endianness();
        self[11].switch_endianness();
        self[12].switch_endianness();
        self[13].switch_endianness();
        self[14].switch_endianness();
        self[15].switch_endianness();
        self[16].switch_endianness();
        self[17].switch_endianness();
        self[18].switch_endianness();
        self[19].switch_endianness();
        self[20].switch_endianness();
        self[21].switch_endianness();
        self[22].switch_endianness();
        self[23].switch_endianness();
        self[24].switch_endianness();
        self[25].switch_endianness();
        self[26].switch_endianness();
        self[27].switch_endianness();
        self[28].switch_endianness();
        self[29].switch_endianness();
        self[30].switch_endianness();
        self[31].switch_endianness();
    }
}

/// `Unpacker` provides the `std::io::Read` API but for any type `T` implementing 
/// [`Packed`](trait.Packed.html). It does not perform any endianness conversion and thus always 
/// reads data using the system endianness.
///
/// # Example
/// 
/// Example of reading a file containing a few float samples.
/// 
/// ```no_run
/// use std::fs::File;
/// use std::iter::repeat;
///
/// use bytepack::Unpacker;
///
/// fn read_samples(file: &str) -> Vec<f32> {
///     let mut file = File::open(file).unwrap();
///     let num_samples : u32 = file.unpack().unwrap();
///     let mut samples : Vec<f32> = repeat(0f32).take(num_samples as usize).collect();
///     file.unpack_exact(&mut samples[..]).unwrap();
///     return samples;
/// }
/// ```
pub trait Unpacker {

    /// Unpack a single value of type `T`.
    ///
    /// ```no_run
    /// # use bytepack::Unpacker;
    /// # use std::fs::File;
    /// let mut file = File::open("test").unwrap();
    /// let float : f32 = file.unpack().unwrap();
    /// ```
    fn unpack<T: Packed>(&mut self) -> Result<T>;

    /// Unpack values of type `T` until `EOF` is reached and place them in `buf`. An error is 
    /// returned if the number of bytes read is not a multiple of the size of `T`.
    ///
    /// ```no_run
    /// # use bytepack::Unpacker;
    /// # use std::fs::File;
    /// let mut file = File::open("test").unwrap();
    /// let mut buffer = Vec::<u64>::new();
    /// file.unpack_to_end(&mut buffer).unwrap();
    /// ```
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;

    /// Unpack the exact number of values of type `T` to fill `buf`. An error is 
    /// returned if not enough byte could be read.
    ///
    /// ```no_run
    /// # use bytepack::Unpacker;
    /// # use std::fs::File;
    /// let mut file = File::open("test").unwrap();
    /// let mut buffer = vec![0i32; 10];
    /// file.unpack_exact(&mut buffer[..]).unwrap();
    /// ```
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

/// `Packer` provides the `std::io::Write` API but for any type `T` implementing 
/// [`Packed`](trait.Packed.html). It does not perform any endianness conversion and thus always 
/// writes data using the system endianness.
///
/// # Example
/// 
/// Example of writing a file containing a few float samples.
/// 
/// ```no_run
/// use std::fs::File;
/// use std::iter::repeat;
///
/// use bytepack::Packer;
///
/// fn write_samples(file: &str, samples: &Vec<f32>) {
///     let mut file = File::create(file).unwrap();
///     file.pack(samples.len() as u32).unwrap();
///     file.pack_all(&samples[..]).unwrap();
/// }
/// ```
pub trait Packer {

    /// Pack a single value of type `T`.
    ///
    /// ```no_run
    /// # use bytepack::Packer;
    /// # use std::fs::File;
    /// let mut file = File::create("test").unwrap();
    /// file.pack(42f32).unwrap();
    /// ```
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;

    /// Pack all the values of type `T` from `buf`.
    ///
    /// ```no_run
    /// # use bytepack::Packer;
    /// # use std::fs::File;
    /// let mut file = File::create("test").unwrap();
    /// let mut float_buffer = vec![666u16; 10];
    /// file.pack_all(&mut float_buffer[..]).unwrap();
    /// ```
    fn pack_all<T: Packed>(&mut self, buf: &[T]) -> Result<()>;
}

impl<R> Unpacker for R where R: Read {
    fn unpack<T: Packed>(&mut self) -> Result<T> {
        let mut res: T;
        // safe because we build a slice of exactly size_of::<T> bytes
        unsafe {
            res = zeroed();
            self.read_exact(slice::from_raw_parts_mut(transmute::<&mut T, *mut u8>(&mut res), size_of::<T>()))?;
        }
        return Ok(res);
    }

    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize> {
        // safe because converted is always forgotten before returning, capacity and length are 
        // always recomputed, in case of error buf is truncated to it's original data.
        unsafe {
            let length = buf.len();
            let capacity = buf.capacity();
            let mut converted = Vec::<u8>::from_raw_parts(buf.as_mut_ptr() as *mut u8, length * size_of::<T>(), capacity * size_of::<T>());
            match self.read_to_end(&mut converted) {
                Ok(size) => {
                    if converted.len() % size_of::<T>() != 0 {
                        converted.truncate(length * size_of::<T>());
                        let new_capacity = converted.len() / size_of::<T>();
                        *buf = Vec::from_raw_parts(converted.as_mut_ptr() as *mut T, length, new_capacity);
                        forget(converted);
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof, 
                            format!("read_to_end() returned a number of bytes ({}) which is not a multiple of the size of T ({})", size, size_of::<T>())
                        ));
                    }
                },
                Err(e) => {
                    converted.truncate(length * size_of::<T>());
                    let new_capacity = converted.len() / size_of::<T>();
                    *buf = Vec::from_raw_parts(converted.as_mut_ptr() as *mut T, length, new_capacity);
                    forget(converted);
                    return Err(e);
                }
            };
            let new_length = converted.len() / size_of::<T>();
            let new_capacity = converted.len() / size_of::<T>();
            *buf = Vec::from_raw_parts(converted.as_mut_ptr() as *mut T, new_length, new_capacity);
            forget(converted);
            return Ok(new_length - length);
        }
    }

    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()> {
        // safe because we build a slice of exactly buf.len() * size_of::<T> bytes
        unsafe {
            self.read_exact(slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * size_of::<T>()))
        }
    }
}

impl<W> Packer for W where W: Write {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()> {
        // safe because we build a slice of exactly size_of::<T> bytes
        unsafe {
            self.write_all(slice::from_raw_parts(transmute::<&T, *const u8>(&t), size_of::<T>()))?;
        }
        return Ok(());
    }

    fn pack_all<T: Packed>(&mut self, t: &[T]) -> Result<()> {
        // safe because we build a slice of exactly t.len() * size_of::<T> bytes
        unsafe {
            self.write_all(slice::from_raw_parts(transmute::<*const T, *const u8>(t.as_ptr()), t.len() * size_of::<T>()))?;
        }
        return Ok(());
    }
}

/// Provides the same API and functionnality as [`Unpacker`](trait.Unpacker.html) but ensure that 
/// the data is in little endian format. See [`Unpacker`](trait.Unpacker.html) for more 
/// documentation.
pub trait LEUnpacker {
    fn unpack<T: Packed>(&mut self) -> Result<T>;
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

/// Provides the same API and functionnality as [`Packer`](trait.Packer.html) but ensure that 
/// the data is in little endian format. See [`Packer`](trait.Packer.html) for more 
/// documentation.
pub trait LEPacker {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;

    /// Here T needs to be `Clone` because the endianness switch cannot be done in-place. This method 
    /// thus allocates a copy of `buf` if an endianness switch is needed.
    fn pack_all<T: Packed + Clone>(&mut self, buf: &[T]) -> Result<()>;
}

impl<R> LEUnpacker for R where R: Read {
    fn unpack<T: Packed>(&mut self) -> Result<T> {
        if cfg!(target_endian = "big") {
            let mut t = Unpacker::unpack::<T>(self)?;
            t.switch_endianness();
            Ok(t)
        }
        else {
            Unpacker::unpack(self)
        }
    }

    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize> {
        if cfg!(target_endian = "big") {
            let size = Unpacker::unpack_to_end(self, buf)?;
            let start = buf.len() - size;
            for i in start..buf.len() {
                buf[i].switch_endianness();
            }
            Ok(size)
        }
        else {
            Unpacker::unpack_to_end(self, buf)
        }
    }

    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()> {
        if cfg!(target_endian = "big") {
            Unpacker::unpack_exact(self, buf)?;
            for i in 0..buf.len() {
                buf[i].switch_endianness();
            }
            Ok(())
        }
        else {
            Unpacker::unpack_exact(self, buf)
        }
    }
}

impl<W> LEPacker for W where W: Write {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()> {
        if cfg!(target_endian = "big") {
            let mut t_copy = t;
            t_copy.switch_endianness();
            Packer::pack(self, t_copy)
        }
        else {
            Packer::pack(self, t)
        }
    }

    fn pack_all<T: Packed + Clone>(&mut self, buf: &[T]) -> Result<()> {
        if cfg!(target_endian = "big") {
            let mut buf_copy = buf.to_vec();
            for i in 0..buf_copy.len() {
                buf_copy[i].switch_endianness();
            }
            Packer::pack_all(self, &buf_copy[..])
        }
        else {
            Packer::pack_all(self, buf)
        }
    }
}

/// Provides the same API and functionnality as [`Unpacker`](trait.Unpacker.html) but ensure that 
/// the data is in big endian format. See [`Unpacker`](trait.Unpacker.html) for more 
/// documentation.
pub trait BEUnpacker {
    fn unpack<T: Packed>(&mut self) -> Result<T>;
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

/// Provides the same API and functionnality as [`Packer`](trait.Packer.html) but ensure that 
/// the data is in big endian format. See [`Packer`](trait.Packer.html) for more 
/// documentation.
pub trait BEPacker {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;

    /// Here T needs to be `Clone` because the endianness switch cannot be done in-place. This method 
    /// thus allocates a copy of `buf` if an endianness switch is needed.
    fn pack_all<T: Packed + Clone>(&mut self, buf: &[T]) -> Result<()>;
}

impl<R> BEUnpacker for R where R: Read {
    fn unpack<T: Packed>(&mut self) -> Result<T> {
        if cfg!(target_endian = "big") {
            let mut t = Unpacker::unpack::<T>(self)?;
            t.switch_endianness();
            Ok(t)
        }
        else {
            Unpacker::unpack(self)
        }
    }

    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize> {
        if cfg!(target_endian = "big") {
            let size = Unpacker::unpack_to_end(self, buf)?;
            let start = buf.len() - size;
            for i in start..buf.len() {
                buf[i].switch_endianness();
            }
            Ok(size)
        }
        else {
            Unpacker::unpack_to_end(self, buf)
        }
    }

    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()> {
        if cfg!(target_endian = "big") {
            Unpacker::unpack_exact(self, buf)?;
            for i in 0..buf.len() {
                buf[i].switch_endianness();
            }
            Ok(())
        }
        else {
            Unpacker::unpack_exact(self, buf)
        }
    }
}

impl<W> BEPacker for W where W: Write {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()> {
        if cfg!(target_endian = "big") {
            let mut t_copy = t;
            t_copy.switch_endianness();
            Packer::pack(self, t_copy)
        }
        else {
            Packer::pack(self, t)
        }
    }

    fn pack_all<T: Packed + Clone>(&mut self, buf: &[T]) -> Result<()> {
        if cfg!(target_endian = "big") {
            let mut buf_copy = buf.to_vec();
            for i in 0..buf_copy.len() {
                buf_copy[i].switch_endianness();
            }
            Packer::pack_all(self, &buf_copy[..])
        }
        else {
            Packer::pack_all(self, buf)
        }
    }
}
