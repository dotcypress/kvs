# kvs

🚧 Work in progress.

`[no_std]` Key-Value Store backed by RAM/SRAM/FRAM/MRAM/EEPROM, intended to use in resource-constrained environments.

## Features/Limitations

* Capacity: 1-65400 records, no dynamic resizing
* RAM overhead: zero bytes for read-only store or 16 bytes per allocation slot
* Storage overhead: 8 bytes per bucket
* Max key size: 128 bytes
* Max value size: 32 KB
* Max store size: 16 MB

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.s
