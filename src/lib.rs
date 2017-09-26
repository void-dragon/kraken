//!
//! # Kraken API
//!
//! API implementation for the [Kraken](https://www.kraken.com/) market-place.
//!
//! **Please Donate**
//!
//! + **ETC:** 0x7bC5Ff6Bc22B4C6Af135493E6a8a11A62D209ae5
//! + **XMR:** 49S4VziJ9v2CSkH6mP9km5SGeo3uxhG41bVYDQdwXQZzRF6mG7B4Fqv2aNEYHmQmPfJcYEnwNK1cAGLHMMmKaUWg25rHnkm
//!
//! + https://www.kraken.com/en-us/help/api
//!
extern crate base64;
extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;


use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::{Sha256, Sha512};
use std::collections::HashMap;
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;


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
    pub unixtime: i64,
    pub rfc1123: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Asset {
    pub aclass: String,
    pub altname: String,
    pub decimals: u32,
    pub display_decimals: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Tick {
    pub a: Vec<String>,
    pub b: Vec<String>,
    pub c: Vec<String>,
    pub v: Vec<String>,
    pub p: Vec<String>,
    pub t: Vec<u32>,
    pub l: Vec<String>,
    pub h: Vec<String>,
    pub o: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AssetPair {
    pub altname: String,
    pub aclass_base: String,
    pub base: String,
    pub aclass_quote: String,
    pub quote: String,
    pub lot: String,
    pub pair_decimals: u32,
    pub lot_decimals: u32,
    pub lot_multiplier: u32,
    pub leverage_buy: Vec<u32>,
    pub leverage_sell: Vec<u32>,
    pub fees: Vec<(u64, f64)>,
    pub fees_maker: Option<Vec<(u64, f64)>>,
    pub fee_volume_currency: String,
    pub margin_call: u32,
    pub margin_stop: u32,
}

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

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenOrders {
    pub open: HashMap<String, OrderInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClosedOrders {
    pub closed: HashMap<String, OrderInfo>,
    pub count: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KrakenResult<T> {
    pub error: Vec<String>,
    pub result: Option<T>,
}

pub struct Kraken {
    core: Core,
    client: Client<::hyper_tls::HttpsConnector<::hyper::client::HttpConnector>>,
}

impl Kraken {
    pub fn new() -> Kraken {
        let core = Core::new().unwrap();
        let client = Client::configure()
            .connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

        Kraken {
            client: client,
            core: core,
        }
    }


    fn public(&mut self, url: &str) -> Result<::hyper::Chunk, ::hyper::Error> {
        let work = self.client
            .get(
                format!("https://api.kraken.com/0/public/{}", url)
                    .parse()
                    .unwrap(),
            )
            .and_then(|res| res.body().concat2());

        self.core.run(work)
    }

    ///
    /// Server's time.
    ///
    /// # Note
    ///
    /// This is to aid in approximating the skew time between the server and client.
    ///
    pub fn time(&mut self) -> Result<KrakenResult<Time>, String> {
        self.public("Time")
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// Returns an array of asset names and their info.
    ///
    pub fn assets(&mut self) -> Result<KrakenResult<HashMap<String, Asset>>, String> {
        self.public("Assets")
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })

    }

    ///
    /// Returns an array of pair names and theif info.
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
    pub fn asset_pairs(&mut self) -> Result<KrakenResult<HashMap<String, AssetPair>>, String> {
        self.public("AssetPairs")
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// Returns an array of pair names and their ticker info.
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
    pub fn ticker(
        &mut self,
        pairs: &str,
    ) -> Result<KrakenResult<HashMap<String, Tick>>, String> {
        self.public(&format!("Ticker?pair={}", pairs))
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// # Note
    ///
    /// the last entry in the OHLC array is for the current, not-yet-committed frame and will always be present, 
    /// regardless of the value of "since".
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
    /// <time>, <open>, <high>, <low>, <close>, <vwap>, <volume>, <count>
    ///
    pub fn ohlc(&mut self, pair: &str) -> Result<KrakenResult<OHLC>, String> {
        self.public(&format!("OHLC?pair={}", pair))
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// Get the order depth.
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
    pub fn order_book(&mut self, pair: &str, count: Option<u32>) -> Result<KrakenResult<Depth>, String> {
        let mut url = format!("Depth?pair={}", pair);

        if let Some(ct) = count {
            url = format!("{}&count={}", url, ct);
        }

        self.public(&url)
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

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
    pub fn recent_trades(&mut self, pair: &str, since: Option<&str>) -> Result<KrakenResult<HashMap<String, serde_json::Value>>, String> {
        let mut url = format!("Trades?pair={}", pair);

        if let Some(ct) = since {
            url = format!("{}&since={}", url, ct);
        }

        self.public(&url)
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

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
    pub fn recent_spread(&mut self, pair: &str, since: Option<u32>) -> Result<KrakenResult<HashMap<String, serde_json::Value>>, String> {
        let mut url = format!("Spread?pair={}", pair);

        if let Some(ct) = since {
            url = format!("{}&since={}", url, ct);
        }

        self.public(&url)
            .map_err(|e| format!("{:?}", e))
            .and_then(|data| {
                serde_json::from_slice(&data).map_err(|e| format!("{:?}", e))
            })
    }

    // ----

    fn private(
        &mut self,
        account: &Account,
        method: &str,
        params: &mut HashMap<String, String>,
    ) -> Result<::hyper::Chunk, ::hyper::Error> {
        let path = format!("/0/private/{}", method);
        let url = format!("https://api.kraken.com{}", path);
        let timestamp = ::std::time::UNIX_EPOCH.elapsed().unwrap();
        let nonce = format!("{}{}", timestamp.as_secs(), timestamp.subsec_nanos() / 1000);

        params.insert("nonce".to_owned(), nonce.clone());

        let mut body = params.iter().fold(
            String::new(),
            |data, item| data + item.0 + "=" + item.1 + "&",
        );
        body.pop();

        let secret = base64::decode(&account.secret).unwrap();
        let mut hmac = Hmac::new(Sha512::new(), &secret);
        let mut body_hasher = Sha256::new();

        body_hasher.input(nonce.as_bytes());
        body_hasher.input(body.as_bytes());

        hmac.input(path.as_bytes());
        let mut out: [u8; 32] = [0; 32];
        body_hasher.result(&mut out);
        hmac.input(&out);

        let sign = base64::encode(hmac.result().code());

        let mut req = ::hyper::Request::new(::hyper::Method::Post, url.parse().unwrap());

        {
            let headers = req.headers_mut();

            headers.set_raw("API-Key", vec![account.key.as_bytes().to_vec()]);
            headers.set_raw("API-Sign", vec![sign.as_bytes().to_vec()]);
        }

        req.set_body(body);

        let work = self.client.request(req).and_then(
            |res| res.body().concat2(),
        );

        self.core.run(work)
    }

    ///
    /// Returns an array of asset names and balance amount.
    ///
    pub fn balance(
        &mut self,
        account: &Account,
    ) -> Result<KrakenResult<HashMap<String, String>>, String> {
        let mut params = HashMap::new();
        self.private(account, "Balance", &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|r| {
                serde_json::from_slice(&r).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// + `eb` = equivalent balance (combined balance of all currencies)
    /// + `tb` = trade balance (combined balance of all equity currencies)
    /// + `m` = margin amount of open positions
    /// + `n` = unrealized net profit/loss of open positions
    /// + `c` = cost basis of open positions
    /// + `v` = current floating valuation of open positions
    /// + `e` = equity = trade balance + unrealized net profit/loss
    /// + `mf` = free margin = equity - initial margin (maximum margin available to open new positions)
    /// + `ml` = margin level = (equity / initial margin) * 100
    ///
    pub fn trade_balance(&mut self, account: &Account, aclass: Option<&str>, asset: Option<&str>) -> Result<KrakenResult<TradeBalance>, String> {
        let mut params = HashMap::new();

        if let Some(ct) = aclass {
            params.insert("aclass".to_owned(), String::from(ct));
        }

        if let Some(ct) = asset {
            params.insert("asset".to_owned(), String::from(ct));
        }

        self.private(account, "TradeBalance", &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|r| {
                serde_json::from_slice(&r).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// trades = whether or not to include trades in output (optional.  default = false)
    /// userref = restrict results to given user reference id (optional)
    ///
    /// # Note
    /// 
    /// Unless otherwise stated, costs, fees, prices, and volumes are in the asset pair's scale, 
    /// not the currency's scale. For example, if the asset pair uses a lot size that has a scale of 8, 
    /// the volume will use a scale of 8, even if the currency it represents only has a scale of 2. 
    /// Similarly, if the asset pair's pricing scale is 5, the scale will remain as 5, 
    /// even if the underlying currency has a scale of 8.
    ///
    pub fn open_orders(&mut self, account: &Account, trades: Option<bool>, userref: Option<&str>) -> Result<KrakenResult<OpenOrders>, String> {
        let mut params = HashMap::new();

        if let Some(ct) = trades {
            params.insert("trades".to_owned(), if ct { String::from("true") } else { String::from("false") });
        }

        if let Some(ct) = userref {
            params.insert("userref".to_owned(), String::from(ct));
        }

        self.private(account, "OpenOrders", &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|r| {
                serde_json::from_slice(&r).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    /// # Note 
    /// 
    /// Times given by order tx ids are more accurate than unix timestamps. 
    /// If an order tx id is given for the time, the order's open time is used.
    ///
    pub fn closed_orders(&mut self, account: &Account) -> Result<KrakenResult<ClosedOrders>, String> {
        let mut params = HashMap::new();

        self.private(account, "ClosedOrders", &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|r| {
                serde_json::from_slice(&r).map_err(|e| format!("{:?}", e))
            })
    }

    ///
    ///
    ///
    pub fn add_order(
        &mut self,
        account: &Account,
        pair: &str,
        amount: f64,
        rate: f64,
        ordertype: &str,
    ) -> Result<KrakenResult<HashMap<String, String>>, String> {
        let mut params = HashMap::new();

        params.insert("pair".to_owned(), String::from(pair));
        params.insert("price".to_owned(), format!("{}", rate));
        params.insert("ordertype".to_owned(), String::from(ordertype));
        params.insert("volume".to_owned(), format!("{}", amount));

        self.private(account, "AddOrder", &mut params)
            .map_err(|e| format!("{:?}", e))
            .and_then(|r| {
                serde_json::from_slice(&r).map_err(|e| format!("{:?}", e))
            })
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn get_time() {
        let mut kraken = super::Kraken::new();

        kraken.time().unwrap();
    }

    #[test]
    fn get_assets() {
        let mut kraken = super::Kraken::new();

        kraken.assets().unwrap();
    }

    #[test]
    fn get_asset_pairs() {
        let mut kraken = super::Kraken::new();

        kraken.asset_pairs().unwrap();
    }

    #[test]
    fn get_ticker() {
        let mut kraken = super::Kraken::new();

        kraken.ticker("XETHZEUR").unwrap();
    }

    #[test]
    fn get_ohlc() {
        let mut kraken = super::Kraken::new();

        let ohlc = kraken.ohlc("XETHZEUR").unwrap();

        if ohlc.error.len() > 0 {
            panic!("{:?}", ohlc.error);
        }
    }

    #[test]
    fn get_recent_trades() {
        let mut kraken = super::Kraken::new();

        let data = kraken.recent_trades("XETHZEUR", None).unwrap();

        if data.error.len() > 0 {
            panic!("{:?}", data.error);
        }
    }

    #[test]
    fn get_recent_spread() {
        let mut kraken = super::Kraken::new();

        let data = kraken.recent_spread("XETHZEUR", None).unwrap();

        if data.error.len() > 0 {
            panic!("{:?}", data.error);
        }
    }
}
