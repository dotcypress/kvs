# kvs

[PoC] Key-Value Store backed by SRAM/FRAM/MRAM/EEPROM.

## Limitations

* Store capacity: 32-4096 records
* Max key size: 250 bytes
* Max value size: 64 kilobytes
* Store adapter must support partial page write and multipage read.

## Store memory layout

magic | ver | cap | index     | records
------|-----|-----|-----------|----------
  32  |  8  |  16 | ref * cap | rec * cap

### Record reference layout

hash  | deleted | in use | address
------|---------|--------|--------
  14  |    1    |   1    |  32

### Record layout

vcap | vlen | klen | key      | value
-----|------|------|----------|---------
 16  |  16  |   8  | klen * 8 | vcap * 8

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
