# kvs

`[no_std]` Key-Value Store with small memory footprint, intended to use in resource-constrained environments.


## Limitations

* Store adapter must support partial page write and multipage read.
* Max capacity: 65535 records
* Max key size: 128 bytes
* Max value size: 64 kilobytes

## API

* insert
* insert_with_capacity
* patch
* append

* contains_key
* keys

* load
* load_with_offset

* remove

## Store header memory layout

magic | version |  seed   | capacity | index
------|---------|---------|----------|-----------------------
  32  |    8    |    8    |    16    | CAPACITY * rec_ref_len

### Record reference layout

 hash | fingerprint | size | addr
------|-------------|------|------
  16  |     16      |  16  |  32

sectors?

### Record layout

key_len |     key     | val_len | val_cap |   val
--------|-------------|---------|---------|-------------
   8    | key_len * 8 |   16    |   16    | val_cap * 8

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
