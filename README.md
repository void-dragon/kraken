# Kraken API

Full API implementation for the [Kraken](https://www.kraken.com/) market-place.

**Please Donate**

+ **BTC:** 17voJDvueb7iZtcLRrLtq3dfQYBaSi2GsU
+ **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
+ **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm

**Kraken API Documentation:** https://www.kraken.com/en-us/help/api

**Documentation:**  https://docs.rs/kraken/ ![](https://docs.rs/kraken/badge.svg)

## Example

```rust
extern crate kraken;

fn main() {
  let account = kraken::Account {
    key: String::from("<your-key>"),
    secret: String::from("<your-secret>"),
  };

  let balances = kraken::balance(&account).expect("could not get balance");

  println!("{:?}", balances);

  let tick = kraken::ticker("XETHZUSD").expect("could not get tick");

  println!("{:?}", tick.get("XETHZUSD").a[1].parse::<f64>());

  // ticker all pairs at once :D

  let pairs = kraken.asset_pairs().expect("could not optain kraken pairs");

  let pair_data = pairs.result.unwrap();
  let pairs: Vec<&String> = pair_data.keys().collect();
  let mut pairchain = pairs.iter().fold(
    String::new(),
    |data, item| data + item + ",",
  );
  pairchain.pop();

  kraken::ticker(&pairchain).and_then(|tick| {
    // do funky stuff with a tick
  });
}
```
