# kvs

🚧 Work in progress.

`[no_std]` Key-Value Store backed by RAM/SRAM/FRAM/MRAM/EEPROM, intended to use in resource-constrained environments.

## Features/Limitations

* Max key size: 256B
* Max value size: 64KB
* Max store size: 16MB
* Max capacity: 65400 buckets
* Storage overhead: 8B per bucket
* RAM overhead: 16B per allocation slot, 0B for read-only store

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
