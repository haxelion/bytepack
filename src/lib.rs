//! `bytepack` is a simple crate which extends the `std::io` API to be able to read and write any 
//! data type in their memory representation. It can be seen as a generalization of the 
//! `std::io::Read` and `std::io::Write` trait , but operating on a generic parameter `T` instead 
//! of `u8`. This crate focus on performances by beeing no copy (except in one clearly marked case) 
//! and offering methods to read and write arrays.
//! 
//! `bytepack` offers three trait famillies allowing different endianness control. `Unpacker` and 
//! `Packer` read and write data in the endianness of the operating system. `LEUnpacker` and 
//! `LEPacker` always read and write data in little endian while `BEUnpacker` and `BEPacker` do the 
//! same in big endian. They all conform to the same API which is copied from the one of `std::io`.
//! This means switching from one endianness to another can be done by simply bringing a different 
//! trait in scope.
//!
//! Because `bytepack` is not a serialization library, it cannot read and write complex types like 
//! `Vec`, `Rc`, etc. directly from a Reader or to Writer. Indeed those types do not contain the 
//! underlying data directly packed inside but rather hold a reference or a pointer to it. To 
//! identify types which holds their data "packed" together, the `Packed` trait is used. 
//! Additionnaly it provides a in-memory endianness switching method. One can implement this trait 
//! for the data types deemed safe to read and write. An automatic derive for structures made only 
//! of types implemting `Packed` could be implemented in the future.

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::mem::{zeroed, transmute, size_of, forget};
use std::slice;

pub trait Packed {
    fn switch_endianness(&mut self);
}

impl Packed for u8 {
    fn switch_endianness(&mut self) {
        *self = u8::swap_bytes(*self);
    }
}

impl Packed for i8 {
    fn switch_endianness(&mut self) {
        *self = i8::swap_bytes(*self);
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

pub trait Unpacker {
    fn unpack<T: Packed>(&mut self) -> Result<T>;
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

pub trait Packer {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;
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

pub trait LEUnpacker {
    fn unpack<T: Packed>(&mut self) -> Result<T>;
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

pub trait LEPacker {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;
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

pub trait BEUnpacker {
    fn unpack<T: Packed>(&mut self) -> Result<T>;
    fn unpack_to_end<T: Packed>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T: Packed>(&mut self, buf: &mut [T]) -> Result<()>;
}

pub trait BEPacker {
    fn pack<T: Packed>(&mut self, t: T) -> Result<()>;
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

#[cfg(test)]
mod tests;
