# hypercore-id-encoding-rs

Key ids and z-base-32 exactly as npm [`hypercore-id-encoding`](https://github.com/holepunchto/hypercore-id-encoding)
1.3 and [`z32`](https://github.com/holepunchto/z32) 1.1 produce and accept them.

```rust
use hypercore_id_encoding::{decode, encode, z32};

let key = [7u8; 32];
let id = encode(&key); // 52 z-base-32 characters
assert_eq!(decode(&id).unwrap(), key);
assert_eq!(decode(&format!("pear://{id}/app")).unwrap(), key);
assert_eq!(z32::decode(&z32::encode(b"hi")).unwrap(), b"hi");
```

`decode` accepts `pear://<id>[/…]`, 52 z-base-32 characters or 64 hex characters, like JS.
`z32::decode` keeps the JS decoder's output length for every input length, including lengths the
encoder never produces, where missing trailing characters count as zero.

`fixtures/golden.json` comes from the npm modules (`fixtures/gen`); `cargo test` checks every case.

## License

MIT OR Apache-2.0
