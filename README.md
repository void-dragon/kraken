# Kraken API

API implementation for the [Kraken](https://www.kraken.com/) market-place.

**Please Donate**

+ **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
+ **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm

**Kraken API Documentation:** https://www.kraken.com/en-us/help/api

## Example

```rust
extern crate kraken;

fn main() {
  let mut api = kraken::Kraken::new();
  
  let tick = api.ticker("XETHZUSD").unwrap();
  
  println!("{:?}", tick.result.get("XETHZUSD").a[1].parse::<f64>());
}
```
