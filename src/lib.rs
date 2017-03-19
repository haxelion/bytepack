//! `bytepack` is a simple crate which extends the Write and Read trait to be able to read and 
//! write any data type in their memory representation.
//!
//! It can be seen as a generalization of the Read and Write API, but operating on T instead of u8.
//! To the contrary of the `byteorder` crate, it does not provide endianness control although this 
//! might change in the future.
//!
//! This is not a serialization library and reading or writing complex types containing pointers or 
//! reference will result in undefined behavior and crashes. 

use std::io::{Read, Write, Result, Error, ErrorKind};
use std::mem::{zeroed, transmute, size_of, forget};
use std::slice;

pub trait Unpacker {
    fn unpack<T>(&mut self) -> Result<T>;
    fn unpack_to_end<T>(&mut self, buf: &mut Vec<T>) -> Result<usize>;
    fn unpack_exact<T>(&mut self, buf: &mut [T]) -> Result<()>;
}

pub trait Packer {
    fn pack<T>(&mut self, t: T) -> Result<()>;
    fn pack_all<T>(&mut self, buf: &[T]) -> Result<()>;
}

impl<R> Unpacker for R where R: Read {
    fn unpack<T>(&mut self) -> Result<T> {
        let mut res: T;
        // safe because we build a slice of exactly size_of::<T> bytes
        unsafe {
            res = zeroed();
            self.read_exact(slice::from_raw_parts_mut(transmute::<&mut T, *mut u8>(&mut res), size_of::<T>()))?;
        }
        return Ok(res);
    }

    fn unpack_to_end<T>(&mut self, buf: &mut Vec<T>) -> Result<usize> {
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

    fn unpack_exact<T>(&mut self, buf: &mut [T]) -> Result<()> {
        // safe because we build a slice of exactly buf.len() * size_of::<T> bytes
        unsafe {
            self.read_exact(slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, buf.len() * size_of::<T>()))
        }
    }
}

impl<W> Packer for W where W: Write {
    fn pack<T>(&mut self, t: T) -> Result<()> {
        // safe because we build a slice of exactly size_of::<T> bytes
        unsafe {
            self.write_all(slice::from_raw_parts(transmute::<&T, *const u8>(&t), size_of::<T>()))?;
        }
        return Ok(());
    }

    fn pack_all<T>(&mut self, t: &[T]) -> Result<()> {
        // safe because we build a slice of exactly t.len() * size_of::<T> bytes
        unsafe {
            self.write_all(slice::from_raw_parts(transmute::<*const T, *const u8>(t.as_ptr()), t.len() * size_of::<T>()))?;
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests;
