<h1 align="center">store</h1>
<div align="center">
  <strong>
    A dead simple binary (de)serializer.
  </strong>
</div>

<br />

`store` is a dead simple binary (de)serializer utilizing the
[`Serialize`][serde-serialize] and [`Deserialize`][serde-deserialize] traits
provided by `serde`.

It is fully compatible with `std`, `no_std`, and `no_std` + `alloc`.

## Installation

To use `store`, add this to your Cargo.toml:

```toml
[dependencies]
store = "0.1-alpha.1"
```

## Dumping types

`store` can dump types that implement [`Serialize`][serde-serialize] into
mutable byte buffers.

```rust
use serde_derive::Serialize;
use store::Dump;

#[derive(Serialize)]
struct Foo(u32);

fn main() -> store::Result<()> {
    let mut buf = [0; 4];
    let foo = Foo(42);

    foo.dump_into_bytes(&mut buf[..])?;

    Ok(())
}
```

## Loading types

`store` will also decode structures that implement
[`Deserialize`][serde-deserialize] from byte buffers.

```rust
use serde_derive::Deserialize;
use store::Load;

#[derive(Deserialize)]
struct Bar(u32);

fn main() -> store::Result<()> {
    let buf = [0; 4];
    let bar = Bar::load_from_bytes(&buf[..])?;

    Ok(())
}
```

## License

This project is dual-licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Contributing

If you would like to contribute to `store`, experience any issues, or even have
features you would like to see implemented, [new issues][new-issue] and pull
requests are welcomed.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `store` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[new-issue]: https://github.com/rjsberry/store/issues/new
[serde-serialize]: https://docs.rs/serde/latest/serde/trait.Serialize.html
[serde-deserialize]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
