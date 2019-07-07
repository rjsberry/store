//! `store` is a dead simple binary (de)serializer utilizing the
//! [`Serialize`][serde-serialize] and [`Deserialize`][serde-deserialize] traits
//! provided by `serde`.
//!
//! It is fully compatible with `std`, `no_std`, and `no_std` + `alloc`.
//!
//! # Installation
//!
//! To use `store`, add this to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! store = "0.1.0-alpha.3"
//! ```
//!
//! # Dumping types
//!
//! `store` can dump types that implement [`Serialize`][serde-serialize] into
//! mutable byte buffers.
//!
//! ```rust
//! use serde_derive::Serialize;
//! use store::Dump;
//!
//! #[derive(Serialize)]
//! struct Foo(u32);
//!
//! fn main() -> store::Result<()> {
//!     let mut buf = [0; 4];
//!     let foo = Foo(42);
//!
//!     foo.dump_into_bytes(&mut buf[..])?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Loading types
//!
//! `store` will also decode structures that implement
//! [`Deserialize`][serde-deserialize] from byte buffers.
//!
//! ```rust
//! use serde_derive::Deserialize;
//! use store::Load;
//!
//! #[derive(Deserialize)]
//! struct Bar(u32);
//!
//! fn main() -> store::Result<()> {
//!     let buf = [0; 4];
//!     let bar = Bar::load_from_bytes(&buf[..])?;
//!
//!     Ok(())
//! }
//! ```
//!
//! [serde-serialize]: https://docs.rs/serde/latest/serde/trait.Serialize.html
//! [serde-deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html

#![no_std]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

mod de;
mod error;
mod ser;

pub use self::error::{Error, Result};

use byteio::{prelude::*, Writer};
use serde::{Deserialize, Serialize};

/// Dump an object into a byte stream.
pub trait Dump {
    /// Dumps `self` into the byte stream using native endian byte order.
    fn dump_into_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize>;

    /// Dumps `self` into the byte stream using big endian byte order.
    fn dump_into_be_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize>;

    /// Dumps `self` into the byte stream using little endian byte order.
    fn dump_into_le_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize>;
}

impl<T: Serialize> Dump for T {
    fn dump_into_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize> {
        #[cfg(target_endian = "big")]
        let n = self.dump_into_be_bytes(writer)?;
        #[cfg(target_endian = "little")]
        let n = self.dump_into_le_bytes(writer)?;

        Ok(n)
    }

    fn dump_into_be_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize> {
        let fb = crate::ser::FunctionBank::new(
            <Writer<W> as WriteBytesExt>::try_write_i8,
            <Writer<W> as WriteBytesExt>::try_write_i16_be,
            <Writer<W> as WriteBytesExt>::try_write_i32_be,
            <Writer<W> as WriteBytesExt>::try_write_i64_be,
            <Writer<W> as WriteBytesExt>::try_write_u8,
            <Writer<W> as WriteBytesExt>::try_write_u16_be,
            <Writer<W> as WriteBytesExt>::try_write_u32_be,
            <Writer<W> as WriteBytesExt>::try_write_u64_be,
            <Writer<W> as WriteBytesExt>::try_write_f32_be,
            <Writer<W> as WriteBytesExt>::try_write_f64_be,
        );

        let writer = Writer::new(writer);

        let mut serializer = crate::ser::Serializer::new(writer, fb);
        self.serialize(&mut serializer)?;

        let writer = serializer.into_inner();

        Ok(writer.num_bytes_written())
    }

    fn dump_into_le_bytes<W: WriteBytes>(&self, writer: W) -> crate::Result<usize> {
        let fb = crate::ser::FunctionBank::new(
            <Writer<W> as WriteBytesExt>::try_write_i8,
            <Writer<W> as WriteBytesExt>::try_write_i16_le,
            <Writer<W> as WriteBytesExt>::try_write_i32_le,
            <Writer<W> as WriteBytesExt>::try_write_i64_le,
            <Writer<W> as WriteBytesExt>::try_write_u8,
            <Writer<W> as WriteBytesExt>::try_write_u16_le,
            <Writer<W> as WriteBytesExt>::try_write_u32_le,
            <Writer<W> as WriteBytesExt>::try_write_u64_le,
            <Writer<W> as WriteBytesExt>::try_write_f32_le,
            <Writer<W> as WriteBytesExt>::try_write_f64_le,
        );

        let writer = Writer::new(writer);

        let mut serializer = crate::ser::Serializer::new(writer, fb);
        self.serialize(&mut serializer)?;

        let writer = serializer.into_inner();

        Ok(writer.num_bytes_written())
    }
}

/// Load an object from a byte stream.
pub trait Load<'a>
where
    Self: Sized,
{
    /// Loads `Self` from the byte stream using native endian byte order.
    fn load_from_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self>;

    /// Loads `Self` from the byte stream using big endian byte order.
    fn load_from_be_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self>;

    /// Loads `Self` from the byte stream using little endian byte order.
    fn load_from_le_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self>;
}

impl<'a, T: Deserialize<'a>> Load<'a> for T {
    fn load_from_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self> {
        #[cfg(target_endian = "big")]
        let t = T::load_from_be_bytes(reader)?;
        #[cfg(target_endian = "little")]
        let t = T::load_from_le_bytes(reader)?;

        Ok(t)
    }

    fn load_from_be_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self> {
        let fb = crate::de::FunctionBank::new(
            <R as ReadBytesExt>::try_read_i8,
            <R as ReadBytesExt>::try_read_i16_be,
            <R as ReadBytesExt>::try_read_i32_be,
            <R as ReadBytesExt>::try_read_i64_be,
            <R as ReadBytesExt>::try_read_u8,
            <R as ReadBytesExt>::try_read_u16_be,
            <R as ReadBytesExt>::try_read_u32_be,
            <R as ReadBytesExt>::try_read_u64_be,
            <R as ReadBytesExt>::try_read_f32_be,
            <R as ReadBytesExt>::try_read_f64_be,
        );

        let mut deserializer = crate::de::Deserializer::new(reader, fb);
        Self::deserialize(&mut deserializer)
    }

    fn load_from_le_bytes<R: ReadBytes<'a>>(reader: R) -> crate::Result<Self> {
        let fb = crate::de::FunctionBank::new(
            <R as ReadBytesExt>::try_read_i8,
            <R as ReadBytesExt>::try_read_i16_le,
            <R as ReadBytesExt>::try_read_i32_le,
            <R as ReadBytesExt>::try_read_i64_le,
            <R as ReadBytesExt>::try_read_u8,
            <R as ReadBytesExt>::try_read_u16_le,
            <R as ReadBytesExt>::try_read_u32_le,
            <R as ReadBytesExt>::try_read_u64_le,
            <R as ReadBytesExt>::try_read_f32_le,
            <R as ReadBytesExt>::try_read_f64_le,
        );

        let mut deserializer = crate::de::Deserializer::new(reader, fb);
        Self::deserialize(&mut deserializer)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(any(feature = "std", feature = "alloc"))]
    use alloc::vec::Vec;

    use serde_derive::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    struct Foo {
        bar: i16,
        baz: i64,
        qux: f32,
    }

    const FOO_SERIALIZED_LENGTH: usize = 14;

    #[test]
    fn serialize_into_u8_slice_consumed_bytes() {
        let mut buf = [0u8; FOO_SERIALIZED_LENGTH];
        let foo = Foo::default();
        let bytes_written = foo.dump_into_bytes(&mut buf[..]).unwrap();
        assert_eq!(bytes_written, FOO_SERIALIZED_LENGTH);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn serialize_into_u8_vec_consumed_slice() {
        let mut buf = Vec::new();
        let foo = Foo::default();
        let bytes_written = foo.dump_into_bytes(&mut buf).unwrap();
        assert_eq!(bytes_written, FOO_SERIALIZED_LENGTH);
    }
}
