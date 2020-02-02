# kvs

[PoC] Key-Value Store.

## Limitations

* Max key size: 15 bytes
* Max record size: 4095 bytes
* Supported page sizes: 16-128 bits

## Semantics

* Store adapter must support partial page write and multipage read.
* `append` is guaranteed for the last added key only.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.