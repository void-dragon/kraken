//!
//! # Kraken API
//!
//! API implementation for the [Kraken](https://www.kraken.com/) market-place.
//!
//! **Please Donate**
//!
//! + **BTC:** 17voJDvueb7iZtcLRrLtq3dfQYBaSi2GsU
//! + **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
//! + **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm
//!
//! + https://www.kraken.com/en-us/help/api
//!
//! ## Example
//!
//! ```rust
//! extern crate kraken;
//!
//! fn main() {
//!   let account = kraken::Account {
//!     key: String::from("<your-key>"),
//!     secret: String::from("<your-secret>"),
//!   };
//!
//!   let balances = kraken::balance(&account).expect("could not get balance");
//!
//!   println!("{:?}", balances);
//!
//!   let tick = kraken::ticker("XETHZUSD").expect("could not get tick");
//!
//!   println!("{:?}", tick.get("XETHZUSD").a[1].parse::<f64>());
//!
//!   // ticker all pairs at once :D
//!
//!   let pairs = kraken.asset_pairs().expect("could not optain kraken pairs");
//!
//!   let pair_data = pairs.result.unwrap();
//!   let pairs: Vec<&String> = pair_data.keys().collect();
//!   let mut pairchain = pairs.iter().fold(
//!     String::new(),
//!     |data, item| data + item + ",",
//!   );
//!   pairchain.pop();
//!
//!   kraken::ticker(&pairchain).and_then(|tick| {
//!     // do funky stuff with a tick
//!   });
//! }
//! ```
//!
extern crate base64;
extern crate crypto;
extern crate curl;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;


use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::{Sha256, Sha512};
use curl::easy::{Easy, List};
use std::collections::HashMap;
use std::io::Read;


///
/// Representing a key secret pair from kraken.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub key: String,
    pub secret: String,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct Time {
    /// as unix timestamp
    pub unixtime: i64,
    /// as RFC 1123 time format
    pub rfc1123: String,
}

/// A currency asset
#[derive(Deserialize, Serialize, Debug)]
pub struct Asset {
    /// asset class
    pub aclass: String,
    /// alternate name
    pub altname: String,
    /// scaling decimal places for record keeping
    pub decimals: u32,
    /// scaling decimal places for output display
    pub display_decimals: u32,
}

/// Ticker info
#[derive(Deserialize, Serialize, Debug)]
pub struct Tick {
    /// ask array(<price>, <whole lot volume>, <lot volume>)
    pub a: Vec<String>,
    /// bid array(<price>, <whole lot volume>, <lot volume>)
    pub b: Vec<String>,
    /// last trade closed array(<price>, <lot volume>)
    pub c: Vec<String>,
    /// volume array(<today>, <last 24 hours>)
    pub v: Vec<String>,
    /// volume weighted average price array(<today>, <last 24 hours>)
    pub p: Vec<String>,
    /// number of trades array(<today>, <last 24 hours>)
    pub t: Vec<u32>,
    /// low array(<today>, <last 24 hours>)
    pub l: Vec<String>,
    /// high array(<today>, <last 24 hours>)
    pub h: Vec<String>,
    /// today's opening price
    pub o: String,
}

/// Tradable asset pairs
#[derive(Deserialize, Serialize, Debug)]
pub struct AssetPair {
    /// alternate pair name
    pub altname: String,
    /// asset class of base component
    pub aclass_base: String,
    /// asset id of base component
    pub base: String,
    /// asset class of quote component
    pub aclass_quote: String,
    /// asset id of quote component
    pub quote: String,
    /// volume lot size
    pub lot: String,
    /// scaling decimal places for pair
    pub pair_decimals: u32,
    /// scaling decimal places for volume
    pub lot_decimals: u32,
    /// amount to multiply lot volume by to get currency volume
    pub lot_multiplier: u32,
    /// array of leverage amounts available when buying
    pub leverage_buy: Vec<u32>,
    /// array of leverage amounts available when selling
    pub leverage_sell: Vec<u32>,
    /// fee schedule array in [volume, percent fee] tuples
    pub fees: Vec<(u64, f64)>,
    /// maker fee schedule array in [volume, percent fee] tuples (if on maker/taker)
    pub fees_maker: Option<Vec<(u64, f64)>>,
    /// volume discount currency
    pub fee_volume_currency: String,
    /// margin call level
    pub margin_call: u32,
    /// stop-out/liquidation margin level
    pub margin_stop: u32,
}

/// Open High Low Close data
pub type OHLC = HashMap<String, serde_json::Value>;

#[derive(Deserialize, Serialize, Debug)]
pub struct DepthPairTuple(String, String, i64);

#[derive(Deserialize, Serialize, Debug)]
pub struct DepthPair {
    pub asks: Vec<DepthPairTuple>,
    pub bids: Vec<DepthPairTuple>,
}

pub type Depth = HashMap<String, DepthPair>;

#[derive(Deserialize, Serialize, Debug)]
pub struct TradeBalance {
    /// equivalent balance (combined balance of all currencies)
    pub eb: String,
    /// trade balance (combined balance of all equity currencies)
    pub tb: String,
    /// margin amount of open positions
    pub m: String,
    /// unrealized net profit/loss of open positions
    pub n: String,
    /// cost basis of open positions
    pub c: String,
    /// current floating valuation of open positions
    pub v: String,
    /// equity = trade balance + unrealized net profit/loss
    pub e: String,
    /// free margin = equity - initial margin (maximum margin available to open new positions)
    pub mf: String,
    /// margin level = (equity / initial margin) * 100
    pub ml: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderDescription {
    pub leverage: String,
    pub order: String,
    pub ordertype: String,
    pub pair: String,
    pub price: String,
    pub price2: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// order pending book entry
    Pending,
    /// open order
    Open,
    /// closed order
    Closed,
    /// order canceled
    Canceled,
    /// order expired
    Expired,
}

/// General order info object
#[derive(Deserialize, Serialize, Debug)]
pub struct OrderInfo {
    /// unix timestamp of when order was closed
    pub closetm: Option<f64>,
    /// total cost (quote currency unless unless viqc set in oflags)
    pub cost: String,
    pub descr: OrderDescription,
    /// unix timestamp of order end time (or 0 if not set)
    pub expiretm: f64,
    /// total fee (quote currency)
    pub fee: String,
    /// comma delimited list of miscellaneous info:
    /// + stopped = triggered by stop price
    /// + touched = triggered by touch price
    /// + liquidated = liquidation
    /// + partial = partial fill
    pub misc: String,
    /// comma delimited list of order flags:
    /// + viqc = volume in quote currency
    /// + fcib = prefer fee in base currency (default if selling)
    /// + fciq = prefer fee in quote currency (default if buying)
    /// + nompp = no market price protection
    pub oflags: String,
    /// unix timestamp of when order was placed
    pub opentm: f64,
    /// average price (quote currency unless viqc set in oflags)
    pub price: String,
    /// stop price (quote currency, for trailing stops)
    pub stopprice: Option<String>,
    /// triggered limit price (quote currency, when limit based order type triggered)
    pub limitprice: Option<String>,
    /// additional info on status (if any)
    pub reason: Option<String>,
    /// Referral order transaction id that created this order
    pub refid: Option<String>,
    /// unix timestamp of order start time (or 0 if not set)
    pub starttm: f64,
    /// status of order:
    pub status: OrderStatus,
    /// user reference id
    pub userref: Option<String>,
    /// volume of order (base currency unless viqc set in oflags)
    pub vol: String,
    /// volume executed (base currency unless viqc set in oflags)
    pub vol_exec: String,
}

/// Open orders
#[derive(Deserialize, Serialize, Debug)]
pub struct OpenOrders {
    pub open: HashMap<String, OrderInfo>,
}

/// Closed order result
#[derive(Deserialize, Serialize, Debug)]
pub struct ClosedOrders {
    pub closed: HashMap<String, OrderInfo>,
    pub count: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ClosedOrdersConfigCloseTime {
    Open,
    Close,
    Both,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClosedOrdersConfig {
    /// whether or not to include trades in output (optional.  default = false).
    pub trades: Option<bool>,
    /// restrict results to given user reference id (optional).
    pub userref: Option<String>,
    /// starting unix timestamp or order tx id of results (optional.  exclusive).
    pub start: Option<i64>,
    /// ending unix timestamp or order tx id of results (optional.  inclusive).
    pub end: Option<i64>,
    /// result offset.
    pub ofs: Option<u64>,
    /// which time to use (optional).
    pub closetime: Option<ClosedOrdersConfigCloseTime>,
}

/// Cancel order result
#[derive(Deserialize, Serialize, Debug)]
pub struct CanceldOrders {
    /// number of orders canceled
    count: u32,
    /// if set, order(s) is/are pending cancellation
    pending: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KrakenResult<T> {
    pub error: Vec<String>,
    pub result: Option<T>,
}

fn public(url: &str) -> Result<Vec<u8>, String> {
    let mut easy = Easy::new();
    let mut dst = Vec::new();

    easy.url(&format!("https://api.kraken.com/0/public/{}", url))
        .unwrap();

    let result = {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer.perform()
    };

    result.map_err(|e| format!("{:?}", e)).and_then(
        |_x| Ok(dst),
    )
}

///
/// Server's time.
///
/// # Note
///
/// This is to aid in approximating the skew time between the server and client.
///
/// # Result
///
/// ```json
/// {
///     "error":[],
///     "result": {
///         "unixtime": 1507489778,"
///         rfc1123":"Sun,  8 Oct 17 19:09:38 +0000"
///     }
/// }
/// ```
///
pub fn time() -> Result<Time, String> {
    public("Time").and_then(|dst| {
        serde_json::from_slice(&dst)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<Time>| if result.error.len() > 0 {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Returns an array of asset names and their info.
///
pub fn assets() -> Result<HashMap<String, Asset>, String> {
    public("Assets").and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<HashMap<String, Asset>>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })

}

///
/// Returns an array of pair names and theif info.
///
/// # Note
///
/// If an asset pair is on a maker/taker fee schedule,
/// the taker side is given in "fees" and maker side in "fees_maker".
/// For pairs not on maker/taker, they will only be given in "fees".
///
/// # Result
///
/// ```json
/// {
///     "error": [],
///     "result": {
///         "XETHZEUR": {
///             "altname": "ETHEUR",
///             "aclass_base": "currency",
///             "base": "XETH",
///             "aclass_quote": "currency",
///             "quote": "ZEUR",
///             "lot": "unit",
///             "pair_decimals": 5,
///             "lot_decimals": 8,
///             "lot_multiplier": 1,
///             "leverage_buy": [2, 3],
///             "leverage_sell": [2, 3],
///             "fees": [
///                 [0, 0.26],
///                 [50000, 0.24],
///                 [100000, 0.22],
///                 [250000, 0.2],
///                 [500000, 0.18],
///                 [1000000, 0.16],
///                 [2500000, 0.14],
///                 [5000000, 0.12],
///                 [10000000, 0.1]
///             ],
///             "fees_maker": [
///                 [0, 0.16],
///                 [50000, 0.14],
///                 [100000, 0.12],
///                 [250000, 0.1],
///                 [500000, 0.08],
///                 [1000000, 0.06],
///                 [2500000, 0.04],
///                 [5000000, 0.02],
///                 [10000000, 0]
///             ],
///             "fee_volume_currency": "ZUSD",
///             "margin_call": 80,
///             "margin_stop": 40
///         }
///     }
/// }
/// ```
///
pub fn asset_pairs() -> Result<HashMap<String, AssetPair>, String> {
    public("AssetPairs").and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<HashMap<String, AssetPair>>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })
}

///
/// Returns an array of pair names and their ticker info.
///
/// # Arguments
///
/// + `pairs` - comma delimited list of asset pairs to get info on
///
/// # Result
///
/// + `a` ask array(<price>, <whole lot volume>, <lot volume>)
/// + `b` bid array(<price>, <whole lot volume>, <lot volume>)
/// + `c` last trade closed array(<price>, <lot volume>)
/// + `v` volume array(<today>, <last 24 hours>)
/// + `p` volume weighted average price array(<today>, <last 24 hours>)
/// + `t` number of trades array(<today>, <last 24 hours>)
/// + `l` low array(<today>, <last 24 hours>)
/// + `h` high array(<today>, <last 24 hours>)
/// + `o` today's opening price
///
/// ```json
/// {
///     "error": [],
///     "result": {
///         "XETHZEUR": {
///             "a": ["10.27949", "9", "9.000"],
///             "b": ["10.20800", "83", "83.000"],
///             "c": ["10.27949", "2.91843272"],
///             "v": ["32132.14651679", "155901.33932839"],
///             "p": ["10.20578", "10.18520"],
///             "t": [718, 4203],
///             "l": ["10.11669", "9.87000"],
///             "h": ["10.29992", "10.69000"],
///             "o": "10.24950"
///         }
///     }
/// }
/// ```
pub fn ticker(pairs: &str) -> Result<HashMap<String, Tick>, String> {
    public(&format!("Ticker?pair={}", pairs)).and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<HashMap<String, Tick>>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })
}

///
/// # Arguments
///
/// + `pair` - asset pair to get OHLC data for
/// + `interval` - time frame interval in minutes (optional):
///     1 (default), 5, 15, 30, 60, 240, 1440, 10080, 21600
/// + `since` - return committed OHLC data since given id (optional.  exclusive)
///
/// # Note
///
/// the last entry in the OHLC array is for the current, not-yet-committed frame and will always be present,
/// regardless of the value of "since".
///
/// [time], [open], [high], [low], [close], [vwap], [volume], [count]
///
/// ```json
/// {
///     "error": [],
///     "result": {
///         "XETHZEUR": [
///           [1506303540,"283.62","283.65","283.62","283.65","283.64","7.10086462",4],
///           [1506303600,"283.64","284.09","283.64","284.09","283.97","8.14638417",8],
///           ...
///         ]
///     }
/// }
/// ```
///
pub fn ohlc(pair: &str, interval: Option<u32>, since: Option<&str>) -> Result<OHLC, String> {
    let mut url = format!("OHLC?pair={}", pair);

    if let Some(interval) = interval {
        url = format!("{}&interval={}", url, interval);
    }

    if let Some(since) = since {
        url = format!("{}&since={}", url, since);
    }

    public(&url).and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<OHLC>| if result.error.len() > 0 {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Get the order depth.
///
/// # Arguments
///
/// + `pair` - asset pair to get market depth for
/// + `count` - maximum number of asks/bids (optional)
///
/// ```json
/// {
///     "error":[],
///     "result": {
///         "XETHZEUR": {
///             "asks":[["247.00000","45.273",1506366345], ...],
///             "bids":[["247.00000","45.273",1506366345], ...]
///         }
///     }
/// }
/// ```
///
pub fn order_book(pair: &str, count: Option<u32>) -> Result<Depth, String> {
    let mut url = format!("Depth?pair={}", pair);

    if let Some(ct) = count {
        url = format!("{}&count={}", url, ct);
    }

    public(&url).and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<Depth>| if result.error.len() > 0 {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Get recent trades.
///
/// # Arguments
///
/// + `pair` - asset pair to get trade data for
/// + `since` - return trade data since given id (optional.  exclusive)
///
/// ```json
/// {
///     "error":[],
///     "result": {
///         "XETHZEUR": [
///             ["246.20000","0.86500000",1506362463.76,"b","l",""],
///             ...
///         ],
///         "last":"1506367082091136113"
///     }
/// }
/// ```
///
pub fn recent_trades(
    pair: &str,
    since: Option<&str>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let mut url = format!("Trades?pair={}", pair);

    if let Some(ct) = since {
        url = format!("{}&since={}", url, ct);
    }

    public(&url).and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<
                HashMap<
                    String,
                    serde_json::Value,
                >,
            >| if result.error.len() > 0 {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Get recent spread data.
///
/// # Arguments
///
/// + `pair` - asset pair to get spread data for.
/// + `since` - return spread data since given id (optional.  inclusive).
///
/// ```json
/// {
///     "error":[],
///     "result": {
///         "XETHZEUR": [
///             [1506368083,"247.47000","248.52000"],
///             ...
///         ],
///         "last": 1506370285
///     }
/// }
/// ```
///
pub fn recent_spread(
    pair: &str,
    since: Option<u32>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let mut url = format!("Spread?pair={}", pair);

    if let Some(ct) = since {
        url = format!("{}&since={}", url, ct);
    }

    public(&url).and_then(|data| {
        serde_json::from_slice(&data)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<
                HashMap<
                    String,
                    serde_json::Value,
                >,
            >| if result.error.len() > 0 {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

// ----

fn private(
    account: &Account,
    method: &str,
    params: &mut HashMap<String, String>,
) -> Result<Vec<u8>, String> {
    let path = format!("/0/private/{}", method);
    let url = format!("https://api.kraken.com{}", path);
    let timestamp = ::std::time::UNIX_EPOCH.elapsed().unwrap();
    let nonce = format!("{}{}", timestamp.as_secs(), timestamp.subsec_nanos());

    let mut dst = Vec::new();
    let mut easy = Easy::new();

    easy.url(&url).unwrap();
    easy.post(true).unwrap();

    params.insert("nonce".to_owned(), nonce.clone());

    let mut body = params.iter().fold(
        String::new(),
        |data, item| data + item.0 + "=" + item.1 + "&",
    );
    body.pop();

    let mut body_bytes = body.as_bytes();
    let secret = base64::decode(&account.secret).unwrap();
    let mut hmac = Hmac::new(Sha512::new(), &secret);
    let mut body_hasher = Sha256::new();

    body_hasher.input(nonce.as_bytes());
    body_hasher.input(body_bytes);

    hmac.input(path.as_bytes());
    let mut out: [u8; 32] = [0; 32];
    body_hasher.result(&mut out);
    hmac.input(&out);

    let sign = base64::encode(hmac.result().code());

    easy.post_field_size(body_bytes.len() as u64).unwrap();

    let mut list = List::new();

    list.append("Content-Type: application/x-www-form-urlencoded")
        .unwrap();
    list.append(&format!("API-Key: {}", account.key)).unwrap();
    list.append(&format!("API-Sign: {}", sign)).unwrap();

    easy.http_headers(list).unwrap();

    let result = {
        let mut transfer = easy.transfer();

        transfer
            .read_function(|buf| Ok(body_bytes.read(buf).unwrap_or(0)))
            .unwrap();

        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        transfer.perform()
    };

    result.map_err(|e| format!("{:?}", e)).and_then(
        |_x| Ok(dst),
    )
}

///
/// Returns an array of asset names and balance amount.
///
/// # Arguments
///
/// + `account` - The account credentials to use.
///
pub fn balance(account: &Account) -> Result<HashMap<String, String>, String> {
    let mut params = HashMap::new();
    private(account, "Balance", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<HashMap<String, String>>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })
}

///
/// Get trade balance.
///
/// # Arguments
///
/// + `account` - The account credentials to use.
/// + `asset` - class (optional): currency (default).
/// + `asset` = base asset used to determine balance (default = ZUSD).
///
pub fn trade_balance(
    account: &Account,
    aclass: Option<&str>,
    asset: Option<&str>,
) -> Result<TradeBalance, String> {
    let mut params = HashMap::new();

    if let Some(ct) = aclass {
        params.insert("aclass".to_owned(), String::from(ct));
    }

    if let Some(ct) = asset {
        params.insert("asset".to_owned(), String::from(ct));
    }

    private(account, "TradeBalance", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<TradeBalance>| if result.error.len() >
                0
            {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Get open orders.
///
/// # Arguments
///
/// + `trades` - whether or not to include trades in output (optional.  default = false).
/// + `userref` - restrict results to given user reference id (optional).
///
/// # Note
///
/// Unless otherwise stated, costs, fees, prices, and volumes are in the asset pair's scale,
/// not the currency's scale. For example, if the asset pair uses a lot size that has a scale of 8,
/// the volume will use a scale of 8, even if the currency it represents only has a scale of 2.
/// Similarly, if the asset pair's pricing scale is 5, the scale will remain as 5,
/// even if the underlying currency has a scale of 8.
///
pub fn open_orders(
    account: &Account,
    trades: Option<bool>,
    userref: Option<&str>,
) -> Result<OpenOrders, String> {
    let mut params = HashMap::new();

    if let Some(ct) = trades {
        params.insert(
            "trades".to_owned(),
            if ct {
                String::from("true")
            } else {
                String::from("false")
            },
        );
    }

    if let Some(ct) = userref {
        params.insert("userref".to_owned(), String::from(ct));
    }

    private(account, "OpenOrders", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<OpenOrders>| if result.error.len() >
                0
            {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Get closed orders.
///
/// # Arguments
///
/// + `trades` - whether or not to include trades in output (optional.  default = false).
/// + `userref` - restrict results to given user reference id (optional).
/// + `start` - starting unix timestamp or order tx id of results (optional.  exclusive).
/// + `end` - ending unix timestamp or order tx id of results (optional.  inclusive).
/// + `ofs` - result offset.
/// + `closetime` = which time to use (optional)
///     open
///     close
///     both (default)
///
/// # Note
///
/// Times given by order tx ids are more accurate than unix timestamps.
/// If an order tx id is given for the time, the order's open time is used.
///
pub fn closed_orders(
    account: &Account,
    cfg: Option<ClosedOrdersConfig>,
) -> Result<ClosedOrders, String> {
    let mut params = HashMap::new();

    if let Some(cfg) = cfg {

        if let Some(trades) = cfg.trades {
            if trades {
                params.insert("trades".to_owned(), "true".to_owned());
            } else {
                params.insert("trades".to_owned(), "false".to_owned());
            }
        }

        if let Some(userref) = cfg.userref {
            params.insert("userref".to_owned(), userref);
        }

        if let Some(start) = cfg.start {
            params.insert("start".to_owned(), format!("{}", start));
        }

        if let Some(end) = cfg.end {
            params.insert("end".to_owned(), format!("{}", end));
        }

        if let Some(ofs) = cfg.ofs {
            params.insert("ofs".to_owned(), format!("{}", ofs));
        }

        if let Some(closetime) = cfg.closetime {
            let value = match closetime {
                ClosedOrdersConfigCloseTime::Open => "open",
                ClosedOrdersConfigCloseTime::Close => "close",
                ClosedOrdersConfigCloseTime::Both => "both",
            };

            params.insert("closetime".to_owned(), value.to_owned());
        }
    }

    private(account, "ClosedOrders", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(|result: KrakenResult<ClosedOrders>| if result.error.len() >
                0
            {
                Err(format!("{:?}", result.error))
            } else {
                Ok(result.result.unwrap())
            })
    })
}

///
/// Create a new order.
///
/// # Note
///
/// + Prices can be preceded by +, -, or # to signify the price as a relative amount (with the exception of trailing stops,
///   which are always relative).
///   * + adds the amount to the current offered price.
///   * - subtracts the amount from the current offered price.
///   * # will either add or subtract the amount to the current offered price, depending on the type and order type used.
///  Relative prices can be suffixed with a % to signify the relative amount as a percentage of the offered price.
/// + For orders using leverage, 0 can be used for the volume to auto-fill the volume needed to close out your position.
/// + If you receive the error "EOrder:Trading agreement required", refer to your API key management page for further details.
///
pub fn add_order(
    account: &Account,
    pair: &str,
    amount: &str,
    rate: &str,
    ordertype: &str,
) -> Result<HashMap<String, String>, String> {
    let mut params = HashMap::new();

    params.insert("pair".to_owned(), String::from(pair));
    params.insert("price".to_owned(), format!("{}", rate));
    params.insert("ordertype".to_owned(), String::from(ordertype));
    params.insert("volume".to_owned(), format!("{}", amount));

    private(account, "AddOrder", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<HashMap<String, String>>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })
}

///
/// Cancels an order.
///
/// # Note
///
///  txid may be a user reference id.
///
pub fn cancel_order(account: &Account, txid: &str) -> Result<CanceldOrders, String> {
    let mut params = HashMap::new();

    params.insert("txid".to_owned(), String::from(txid));

    private(account, "CancelOrder", &mut params).and_then(|r| {
        serde_json::from_slice(&r)
            .map_err(|e| format!("{:?}", e))
            .and_then(
                |result: KrakenResult<CanceldOrders>| if result.error.len() > 0 {
                    Err(format!("{:?}", result.error))
                } else {
                    Ok(result.result.unwrap())
                },
            )
    })
}
