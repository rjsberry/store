use crate::Error;

use core::convert::{TryFrom, TryInto};
use core::num::TryFromIntError;

use byteio::{prelude::*, Writer};

/// Unsigned LEB128 variable-length encoding.
///
/// This structure is used by `store` internally when serializing sequence
/// lengths, and is provided for completeness.
///
/// It can be (de)constructed using the `From` and `TryFrom` traits.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ULeb128(u64);

impl ULeb128 {
    const MARKER: u8 = 0x80;

    pub(crate) fn _read_from<'a, R: ReadBytes<'a>>(reader: &mut R) -> crate::Result<Self> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let b = reader.try_read_u8()?;

            if shift == 64 && b > 1 {
                return Err(Error::InvalidEncoding);
            }

            result |= u64::from(b & !Self::MARKER) << shift;

            if b & Self::MARKER == 0 {
                return Ok(Self(result));
            }

            shift += 7;
        }
    }

    /// Attempts to read an unsigned LEB128 encoded value from a buffer.
    pub fn read_from<'a, R: ReadBytes<'a>>(mut reader: R) -> crate::Result<Self> {
        Self::_read_from(&mut reader)
    }

    pub(crate) fn _write_into<W: WriteBytes>(mut self, writer: &mut W) -> crate::Result<()> {
        loop {
            let mut b = (self.0 as u8) & !Self::MARKER;
            self.0 >>= 7;

            if self.0 != 0 {
                b |= Self::MARKER;
            }

            writer.try_write_u8(b)?;

            if self.0 == 0 {
                return Ok(());
            }
        }
    }

    /// Attempts to write a value into a buffer using unsigned LEB128 encoding.
    pub fn write_into<W: WriteBytes>(self, writer: W) -> crate::Result<usize> {
        let mut writer = Writer::new(writer);
        self._write_into(&mut writer)?;

        Ok(writer.num_bytes_written())
    }
}

macro_rules! uleb128_from_unsigned {
    ($ty:ty) => {
        impl From<$ty> for ULeb128 {
            fn from(n: $ty) -> Self {
                Self(n.into())
            }
        }
    };
}

uleb128_from_unsigned!(u8);
uleb128_from_unsigned!(u16);
uleb128_from_unsigned!(u32);

impl From<u64> for ULeb128 {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl From<usize> for ULeb128 {
    fn from(n: usize) -> Self {
        Self(n as _)
    }
}

macro_rules! unsigned_try_from_uleb128 {
    ($ty:ty) => {
        impl TryFrom<ULeb128> for $ty {
            type Error = TryFromIntError;

            fn try_from(v: ULeb128) -> Result<Self, Self::Error> {
                v.0.try_into()
            }
        }
    };
}

unsigned_try_from_uleb128!(u8);
unsigned_try_from_uleb128!(u16);
unsigned_try_from_uleb128!(u32);

impl From<ULeb128> for u64 {
    fn from(v: ULeb128) -> Self {
        v.0
    }
}

unsigned_try_from_uleb128!(usize);
