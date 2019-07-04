macro_rules! if_alloc {
    ($( $i:item )+) => {
        $( #[cfg(any(feature = "std", feature = "alloc"))] $i )+
    };
}

macro_rules! validate {
    ($ty:ty, $var:expr, $width:expr, $dump:ident, $load:ident $( , )?) => {{
        let mut buf = [0; $width];
        $var.$dump(&mut buf[..]).unwrap();
        let out = <$ty>::$load(&buf[..]).unwrap();
        &$var == &out
    }};
}

macro_rules! qc {
    ($qc:ident, $ty:ty, $width:expr, $dump:ident, $load:ident $( , )?) => {
        #[quickcheck]
        fn $qc(x: $ty) -> bool {
            validate!($ty, x, $width, $dump, $load)
        }
    };
}

if_alloc! {
    macro_rules! validate_buf {
        ($ty:ty, $var:expr, $dump:ident, $load:ident $( , )?) => {{
            let mut buf = Vec::new();
            $var.$dump(&mut buf).unwrap();
            let out = <$ty>::$load(buf.as_slice()).unwrap();
            &$var == &out
        }};
    }

    macro_rules! qc_vec {
        ($qc:ident, $ty:ty, $dump:ident, $load:ident $( , )?) => {
            #[quickcheck]
            fn $qc(x: Vec<$ty>) -> TestResult {
                TestResult::from_bool(validate_buf!(Vec<$ty>, x, $dump, $load))
            }
        };
    }

    macro_rules! qc_hashmap {
        ($qc:ident, $kty:ty, $vty:ty, $dump:ident, $load:ident $( , )?) => {
            #[quickcheck]
            fn $qc(x: ::std::collections::HashMap<$kty, $vty>) -> TestResult {
                TestResult::from_bool(validate_buf!(
                    ::std::collections::HashMap<$kty, $vty>,
                    x,
                    $dump,
                    $load,
                ))
            }
        }
    }
}

macro_rules! qc_crate {
    ($qc_mod:ident, $dump:ident, $load:ident) => {
        mod $qc_mod {
            use serde_derive::{Deserialize, Serialize};
            use store::{Dump, Load};

            use quickcheck::TestResult;
            use quickcheck_macros::quickcheck;

            qc!(qc_bool, bool, 1, $dump, $load);

            qc!(qc_i8, i8, 1, $dump, $load);
            qc!(qc_i16, i16, 2, $dump, $load);
            qc!(qc_i32, i32, 4, $dump, $load);
            qc!(qc_i64, i64, 8, $dump, $load);

            qc!(qc_u8, u8, 1, $dump, $load);
            qc!(qc_u16, u16, 2, $dump, $load);
            qc!(qc_u32, u32, 4, $dump, $load);
            qc!(qc_u64, u64, 8, $dump, $load);

            qc!(qc_f32, f32, 4, $dump, $load);
            qc!(qc_f64, f64, 8, $dump, $load);

            qc!(qc_char, char, 4, $dump, $load);

            if_alloc! {
                #[quickcheck]
                fn qc_borrowed_str(x: String) -> TestResult {
                    TestResult::from_bool(validate_buf!(&str, &*x, $dump, $load))
                }

                #[quickcheck]
                fn qc_borrowed_bytes(x: Vec<u8>) -> TestResult {
                    TestResult::from_bool(validate_buf!(&[u8], &*x, $dump, $load))
                }
            }

            qc!(qc_option, Option<()>, 1, $dump, $load);

            qc!(qc_unit, (), 0, $dump, $load);

            #[quickcheck]
            fn qc_unit_variant(x: u8) -> TestResult {
        #[rustfmt::skip]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                enum Rainbow { R, O, Y, G, B, I, V };

                let variant = match x {
                    0 => Rainbow::R,
                    1 => Rainbow::O,
                    2 => Rainbow::Y,
                    3 => Rainbow::G,
                    4 => Rainbow::B,
                    5 => Rainbow::I,
                    6 => Rainbow::V,
                    _ => {
                        return TestResult::discard();
                    }
                };

                TestResult::from_bool(validate!(Rainbow, variant, 1, $dump, $load))
            }

            #[quickcheck]
            fn qc_newtype_struct(x: i32) -> bool {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                struct Foo(i32);

                let f = Foo(x);

                validate!(Foo, f, 4, $dump, $load)
            }

            #[quickcheck]
            fn qc_newtype_variant(i: u8, x: i16, y: i32) -> TestResult {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                enum Foo {
                    Bar(i16),
                    Baz(i32),
                }

                let variant = match i {
                    0 => Foo::Bar(x),
                    1 => Foo::Baz(y),
                    _ => {
                        return TestResult::discard();
                    }
                };

                TestResult::from_bool(validate!(Foo, variant, 5, $dump, $load))
            }

            #[quickcheck]
            fn qc_tuple(x: i16, y: i32) -> bool {
                let t = (x, y);

                validate!((i16, i32), t, 7, $dump, $load)
            }

            #[quickcheck]
            fn qc_tuple_struct(x: i16, y: i32) -> bool {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                struct Foo(i16, i32);

                let f = Foo(x, y);

                validate!(Foo, f, 7, $dump, $load)
            }

            #[quickcheck]
            fn qc_tuple_variant(i: u8, x: i16, y: i32, z: i64) -> TestResult {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                enum Foo {
                    Bar(i16, i32, i32),
                    Baz(i16, i64),
                }

                let variant = match i {
                    0 => Foo::Bar(x, y, y),
                    1 => Foo::Baz(x, z),
                    _ => {
                        return TestResult::discard();
                    }
                };

                TestResult::from_bool(validate!(Foo, variant, 11, $dump, $load))
            }

            #[quickcheck]
            fn qc_struct(x: i16, y: i32) -> bool {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                struct Foo {
                    bar: i16,
                    baz: i32,
                }

                let f = Foo { bar: x, baz: y };

                validate!(Foo, f, 6, $dump, $load)
            }

            #[quickcheck]
            fn qc_struct_variant(i: u8, x: i16, y: i32) -> TestResult {
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
                enum Foo {
                    Bar { ham: i16 },
                    Baz { eggs: i32 },
                }

                let variant = match i {
                    0 => Foo::Bar { ham: x },
                    1 => Foo::Baz { eggs: y },
                    _ => {
                        return TestResult::discard();
                    }
                };

                TestResult::from_bool(validate!(Foo, variant, 5, $dump, $load))
            }

            if_alloc! {
                qc_vec!(qc_vec_i8, i8, $dump, $load);
                qc_vec!(qc_vec_i16, i16, $dump, $load);
                qc_vec!(qc_vec_i32, i32, $dump, $load);
                qc_vec!(qc_vec_i64, i64, $dump, $load);
                qc_vec!(qc_vec_u8, u8, $dump, $load);
                qc_vec!(qc_vec_u16, u16, $dump, $load);
                qc_vec!(qc_vec_u32, u32, $dump, $load);
                qc_vec!(qc_vec_u64, u64, $dump, $load);
                qc_vec!(qc_vec_f32, f32, $dump, $load);
                qc_vec!(qc_vec_f64, f64, $dump, $load);
                qc_vec!(qc_vec_string, String, $dump, $load);

                #[quickcheck]
                fn qc_string(x: String) -> TestResult {
                    TestResult::from_bool(validate_buf!(String, x, $dump, $load))
                }

                qc_hashmap!(qc_hashmap_i8_i8, i8, i8, $dump, $load);
                qc_hashmap!(qc_hashmap_i16_i16, i16, i16, $dump, $load);
                qc_hashmap!(qc_hashmap_i32_i32, i32, i32, $dump, $load);
                qc_hashmap!(qc_hashmap_i64_i64, i64, i64, $dump, $load);
                qc_hashmap!(qc_hashmap_u8_u8, u8, u8, $dump, $load);
                qc_hashmap!(qc_hashmap_u16_u16, u16, u16, $dump, $load);
                qc_hashmap!(qc_hashmap_u32_u32, u32, u32, $dump, $load);
                qc_hashmap!(qc_hashmap_u64_u64, u64, u64, $dump, $load);
                qc_hashmap!(qc_hashmap_vec_u8_vec_u8, Vec<u8>, Vec<u8>, $dump, $load);
                qc_hashmap!(qc_hashmap_string_string, String, String, $dump, $load);
            }
        }
    };
}

qc_crate!(le, dump_into_le_bytes, load_from_le_bytes);
qc_crate!(be, dump_into_be_bytes, load_from_be_bytes);

mod uleb128 {
    use store::ULeb128;

    use quickcheck_macros::quickcheck;

    #[test]
    fn unsigned_leb128_wikipedia_ex() {
        let mut buf = Vec::with_capacity(3);
        assert_eq!(ULeb128::from(624485_u64).write_into(&mut buf).unwrap(), 3);
        assert_eq!(buf, vec![0xE5, 0x8E, 0x26]);
        assert_eq!(u64::from(ULeb128::read_from(&*buf).unwrap()), 624485);
    }

    #[quickcheck]
    fn qc_unsigned_leb128(v: u64) -> bool {
        let mut buf = Vec::with_capacity(10);
        ULeb128::from(v).write_into(&mut buf).unwrap();
        u64::from(ULeb128::read_from(&*buf).unwrap()) == v
    }
}
