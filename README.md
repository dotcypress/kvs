# kvs

🚧 Work in progress.

`[no_std]` Key-Value Store backed by RAM/SRAM/FRAM/MRAM/EEPROM with tiny memory footprint, intended to use in resource-constrained environments.

## Features/Limitations

* Storage overhead: 8 bytes per bucket.
* RAM overhead: zero bytes for read-only store or 16 bytes per allocation slot.
* Max capacity: 65,400 records
* Max store size: 16 MB
* Max value size: 32 KB
* Max key size: 128 B

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
