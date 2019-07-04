use crate::ULeb128;

use byteio::WriteBytes;
use serde::Serialize;

pub(crate) struct FunctionBank<W, E> {
    i8fn: fn(&mut W, n: i8) -> Result<(), E>,
    i16fn: fn(&mut W, n: i16) -> Result<(), E>,
    i32fn: fn(&mut W, n: i32) -> Result<(), E>,
    i64fn: fn(&mut W, n: i64) -> Result<(), E>,
    u8fn: fn(&mut W, n: u8) -> Result<(), E>,
    u16fn: fn(&mut W, n: u16) -> Result<(), E>,
    u32fn: fn(&mut W, n: u32) -> Result<(), E>,
    u64fn: fn(&mut W, n: u64) -> Result<(), E>,
    f32fn: fn(&mut W, n: f32) -> Result<(), E>,
    f64fn: fn(&mut W, n: f64) -> Result<(), E>,
}

impl<W, E> FunctionBank<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        i8fn: fn(&mut W, n: i8) -> Result<(), E>,
        i16fn: fn(&mut W, n: i16) -> Result<(), E>,
        i32fn: fn(&mut W, n: i32) -> Result<(), E>,
        i64fn: fn(&mut W, n: i64) -> Result<(), E>,
        u8fn: fn(&mut W, n: u8) -> Result<(), E>,
        u16fn: fn(&mut W, n: u16) -> Result<(), E>,
        u32fn: fn(&mut W, n: u32) -> Result<(), E>,
        u64fn: fn(&mut W, n: u64) -> Result<(), E>,
        f32fn: fn(&mut W, n: f32) -> Result<(), E>,
        f64fn: fn(&mut W, n: f64) -> Result<(), E>,
    ) -> Self {
        Self { i8fn, i16fn, i32fn, i64fn, u8fn, u16fn, u32fn, u64fn, f32fn, f64fn }
    }
}

pub(crate) struct Serializer<W, E> {
    writer: W,
    fb: FunctionBank<W, E>,
}

impl<W, E> Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    pub fn new(writer: W, fb: FunctionBank<W, E>) -> Self {
        Self { writer, fb }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W, E> Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    fn try_serialize_len<T>(&mut self, v: T) -> crate::Result<()>
    where
        T: Into<ULeb128>,
    {
        v.into()._write_into(&mut self.writer)
    }
}

macro_rules! impl_serialize_primitive {
    ( $ser_fn:ident, $wtr_fn:ident, $ty:ty ) => {
        fn $ser_fn(self, v: $ty) -> crate::Result<()> {
            (self.fb.$wtr_fn)(&mut self.writer, v).map_err(Into::into)
        }
    };
}

impl<'r, W, E> ::serde::ser::Serializer for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    type SerializeMap = ::serde::ser::Impossible<(), Self::Error>;
    #[cfg(any(feature = "std", feature = "alloc"))]
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> crate::Result<()> {
        u8::from(v).serialize(self)
    }

    impl_serialize_primitive!(serialize_i8, i8fn, i8);
    impl_serialize_primitive!(serialize_i16, i16fn, i16);
    impl_serialize_primitive!(serialize_i32, i32fn, i32);
    impl_serialize_primitive!(serialize_i64, i64fn, i64);

    impl_serialize_primitive!(serialize_u8, u8fn, u8);
    impl_serialize_primitive!(serialize_u16, u16fn, u16);
    impl_serialize_primitive!(serialize_u32, u32fn, u32);
    impl_serialize_primitive!(serialize_u64, u64fn, u64);

    impl_serialize_primitive!(serialize_f32, f32fn, f32);
    impl_serialize_primitive!(serialize_f64, f64fn, f64);

    fn serialize_char(self, v: char) -> crate::Result<()> {
        u32::from(v).serialize(self)
    }

    fn serialize_str(self, v: &str) -> crate::Result<()> {
        v.as_bytes().serialize(self)
    }

    fn serialize_bytes(self, v: &[u8]) -> crate::Result<()> {
        self.try_serialize_len(v.len())?;
        self.writer.try_write_exact(v)?;
        Ok(())
    }

    fn serialize_none(self) -> crate::Result<()> {
        0_u8.serialize(self)
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> crate::Result<()>
    where
        T: ::serde::ser::Serialize,
    {
        self.serialize_u8(1)?;
        v.serialize(self)
    }

    fn serialize_unit(self) -> crate::Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> crate::Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> crate::Result<()> {
        self.try_serialize_len(variant_index).map_err(|_| crate::Error::TooManyEnumVariants)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> crate::Result<()>
    where
        T: ::serde::ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> crate::Result<()>
    where
        T: Serialize,
    {
        self.try_serialize_len(variant_index).map_err(|_| crate::Error::TooManyEnumVariants)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> crate::Result<Self::SerializeSeq> {
        self.try_serialize_len(len.ok_or(crate::Error::SequencesMustHaveLength)?)?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> crate::Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeTupleVariant> {
        self.try_serialize_len(variant_index).map_err(|_| crate::Error::TooManyEnumVariants)?;
        Ok(self)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn serialize_map(self, len: Option<usize>) -> crate::Result<Self::SerializeMap> {
        self.try_serialize_len(len.ok_or(crate::Error::SequencesMustHaveLength)?)?;
        Ok(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn serialize_map(self, _len: Option<usize>) -> crate::Result<Self::SerializeMap> {
        Err(crate::Error::UnsupportedDataStructure)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStructVariant> {
        self.try_serialize_len(variant_index)?;
        Ok(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn collect_str<T: ?Sized>(self, _value: &T) -> crate::Result<()> {
        Err(crate::Error::UnsupportedDataStructure)
    }
}

impl<'r, W, E> ::serde::ser::SerializeSeq for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'r, W, E> ::serde::ser::SerializeTuple for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'r, W, E> ::serde::ser::SerializeTupleStruct for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'r, W, E> ::serde::ser::SerializeTupleVariant for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'r, W, E> ::serde::ser::SerializeMap for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> crate::Result<()>
    where
        K: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> crate::Result<()>
    where
        V: ::serde::ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'r, W, E> ::serde::ser::SerializeStruct for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

impl<'r, W, E> ::serde::ser::SerializeStructVariant for &'r mut Serializer<W, E>
where
    W: WriteBytes,
    crate::Error: From<E>,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> crate::Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}
