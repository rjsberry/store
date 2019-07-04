use crate::ULeb128;

use core::convert::TryInto;

use byteio::ReadBytes;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{borrow::ToOwned, vec::Vec};

pub(crate) struct FunctionBank<R, E> {
    i8fn: fn(&mut R) -> Result<i8, E>,
    i16fn: fn(&mut R) -> Result<i16, E>,
    i32fn: fn(&mut R) -> Result<i32, E>,
    i64fn: fn(&mut R) -> Result<i64, E>,
    u8fn: fn(&mut R) -> Result<u8, E>,
    u16fn: fn(&mut R) -> Result<u16, E>,
    u32fn: fn(&mut R) -> Result<u32, E>,
    u64fn: fn(&mut R) -> Result<u64, E>,
    f32fn: fn(&mut R) -> Result<f32, E>,
    f64fn: fn(&mut R) -> Result<f64, E>,
}

impl<'a, R, E> FunctionBank<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        i8fn: fn(&mut R) -> Result<i8, E>,
        i16fn: fn(&mut R) -> Result<i16, E>,
        i32fn: fn(&mut R) -> Result<i32, E>,
        i64fn: fn(&mut R) -> Result<i64, E>,
        u8fn: fn(&mut R) -> Result<u8, E>,
        u16fn: fn(&mut R) -> Result<u16, E>,
        u32fn: fn(&mut R) -> Result<u32, E>,
        u64fn: fn(&mut R) -> Result<u64, E>,
        f32fn: fn(&mut R) -> Result<f32, E>,
        f64fn: fn(&mut R) -> Result<f64, E>,
    ) -> Self {
        Self { i8fn, i16fn, i32fn, i64fn, u8fn, u16fn, u32fn, u64fn, f32fn, f64fn }
    }
}

pub(crate) struct Deserializer<R, E> {
    reader: R,
    fb: FunctionBank<R, E>,
}

impl<'a, R, E> Deserializer<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    pub fn new(reader: R, fb: FunctionBank<R, E>) -> Self {
        Self { reader, fb }
    }
}

impl<'a, R, E> Deserializer<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    fn try_deserialize_len(&mut self) -> crate::Result<usize> {
        let v = ULeb128::_read_from(&mut self.reader)?;
        Ok(v.try_into().map_err(|_| crate::Error::SequenceTooLong)?)
    }

    fn try_deserialize_bytes(&mut self) -> crate::Result<&'a [u8]> {
        self.try_deserialize_len()
            .and_then(|len| self.reader.try_read_exact(len).map_err(Into::into))
    }

    fn try_deserialize_str(&mut self) -> crate::Result<&'a str> {
        Ok(::core::str::from_utf8(self.try_deserialize_bytes()?)
            .map_err(|_| crate::Error::InvalidUtf8Encoding)?)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn try_deserialize_vec(&mut self) -> crate::Result<Vec<u8>> {
        Ok(Vec::from(self.try_deserialize_bytes()?))
    }
}

macro_rules! impl_deserialize_primitive {
    ( $de_fn:ident, $visitor_fn:ident, $rdr_fn:ident ) => {
        fn $de_fn<V>(self, visitor: V) -> crate::Result<V::Value>
        where
            V: ::serde::de::Visitor<'de>,
        {
            visitor.$visitor_fn((self.fb.$rdr_fn)(&mut self.reader)?)
        }
    };
}

impl<'r, 'de, 'a: 'r + 'de, R, E> ::serde::de::Deserializer<'de> for &'r mut Deserializer<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    type Error = crate::Error;

    fn deserialize_any<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::Custom { msg: "store does not support 'deserialize_any'" })
    }

    fn deserialize_bool<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let value = (self.fb.u8fn)(&mut self.reader)?;

        match value {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(crate::Error::InvalidEncoding),
        }
    }

    impl_deserialize_primitive!(deserialize_i8, visit_i8, i8fn);
    impl_deserialize_primitive!(deserialize_i16, visit_i16, i16fn);
    impl_deserialize_primitive!(deserialize_i32, visit_i32, i32fn);
    impl_deserialize_primitive!(deserialize_i64, visit_i64, i64fn);

    impl_deserialize_primitive!(deserialize_u8, visit_u8, u8fn);
    impl_deserialize_primitive!(deserialize_u16, visit_u16, u16fn);
    impl_deserialize_primitive!(deserialize_u32, visit_u32, u32fn);
    impl_deserialize_primitive!(deserialize_u64, visit_u64, u64fn);

    impl_deserialize_primitive!(deserialize_f32, visit_f32, f32fn);
    impl_deserialize_primitive!(deserialize_f64, visit_f64, f64fn);

    fn deserialize_char<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let c: u32 = ::serde::de::Deserialize::deserialize(self)?;
        visitor.visit_char(::core::char::from_u32(c).ok_or_else(|| crate::Error::InvalidEncoding)?)
    }

    fn deserialize_str<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.try_deserialize_str()?)
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn deserialize_string<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::UnsupportedDataStructure)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn deserialize_string<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.try_deserialize_str()?.to_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.try_deserialize_bytes()?)
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn deserialize_byte_buf<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::UnsupportedDataStructure)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn deserialize_byte_buf<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.try_deserialize_vec()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let encoding = (self.fb.u8fn)(&mut self.reader)?;

        match encoding {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(&mut *self),
            _ => Err(crate::Error::InvalidEncoding),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let len = self.try_deserialize_len()?;
        visitor.visit_seq(DeserializeSeq::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_seq(DeserializeSeq::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn deserialize_map<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::UnsupportedDataStructure)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn deserialize_map<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        let len = self.try_deserialize_len()?;
        visitor.visit_map(DeserializeSeq::new(self, len))
    }

    fn deserialize_struct<V>(
        self,
        _name: &str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _enum: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::Custom { msg: "store does not support 'deserialize_identifier'" })
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        Err(crate::Error::Custom { msg: "store does not support 'deserialize_ignored_any'" })
    }
}

impl<'r, 'de, 'a: 'r + 'de, R, E> ::serde::de::EnumAccess<'de> for &'r mut Deserializer<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    type Error = crate::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> crate::Result<(V::Value, Self::Variant)>
    where
        V: ::serde::de::DeserializeSeed<'de>,
    {
        use ::serde::de::IntoDeserializer;

        let variant_idx: u32 = ULeb128::_read_from(&mut self.reader)?
            .try_into()
            .map_err(|_| crate::Error::TooManyEnumVariants)?;

        let val: crate::Result<_> = seed.deserialize(variant_idx.into_deserializer());
        let val = val?;

        Ok((val, self))
    }
}

impl<'r, 'de, 'a: 'r + 'de, R, E> ::serde::de::VariantAccess<'de> for &'r mut Deserializer<R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    type Error = crate::Error;

    fn unit_variant(self) -> crate::Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> crate::Result<T::Value>
    where
        T: ::serde::de::DeserializeSeed<'de>,
    {
        ::serde::de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        ::serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: ::serde::de::Visitor<'de>,
    {
        ::serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

struct DeserializeSeq<'r, R, E>(&'r mut Deserializer<R, E>, usize);

impl<'r, 'a: 'r, R, E> DeserializeSeq<'r, R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    fn new(de: &'r mut Deserializer<R, E>, len: usize) -> Self {
        DeserializeSeq(de, len)
    }

    fn next_element_seed_unchecked<'de, T>(&mut self, seed: T) -> crate::Result<T::Value>
    where
        'a: 'de,
        T: ::serde::de::DeserializeSeed<'de>,
    {
        T::deserialize(seed, &mut *self.0)
    }

    fn next_element_seed<'de, T>(&mut self, seed: T) -> crate::Result<Option<T::Value>>
    where
        'a: 'de,
        T: ::serde::de::DeserializeSeed<'de>,
    {
        if self.1 > 0 {
            self.1 -= 1;
            Ok(Some(self.next_element_seed_unchecked(seed)?))
        } else {
            Ok(None)
        }
    }
}

impl<'r, 'de, 'a: 'r + 'de, R, E> ::serde::de::SeqAccess<'de> for DeserializeSeq<'r, R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    type Error = crate::Error;

    fn next_element_seed<V>(&mut self, seed: V) -> crate::Result<Option<V::Value>>
    where
        'a: 'de,
        V: ::serde::de::DeserializeSeed<'de>,
    {
        self.next_element_seed(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.1)
    }
}

impl<'r, 'de, 'a: 'r + 'de, R, E> ::serde::de::MapAccess<'de> for DeserializeSeq<'r, R, E>
where
    R: ReadBytes<'a>,
    crate::Error: From<E>,
{
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> crate::Result<Option<K::Value>>
    where
        K: ::serde::de::DeserializeSeed<'de>,
    {
        self.next_element_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> crate::Result<V::Value>
    where
        V: ::serde::de::DeserializeSeed<'de>,
    {
        self.next_element_seed_unchecked(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.1)
    }
}
