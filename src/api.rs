use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use ureq;

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub enum QuoteState {
    POSITIVE,
    NEGATIVE,
    NEUTRAL,
}

#[derive(Debug, Clone)]
pub struct Quote {
    timestamp: SystemTime,
    ticker: String,
    price: String,
    price_raw: f64,
    percent_change: String,
    amount_change: String,
    amount_change_raw: f64,
}

impl Quote {
    pub fn from_json(json: &Value) -> Quote {
        Quote {
            timestamp: SystemTime::now(),
            ticker: json["quoteSummary"]["result"][0]["price"]["symbol"]
                .as_str()
                .unwrap_or("err")
                .to_string(),
            price: json["quoteSummary"]["result"][0]["price"]["regularMarketPrice"]["fmt"]
                .as_str()
                .unwrap_or("err")
                .to_string(),
            price_raw: json["quoteSummary"]["result"][0]["price"]["regularMarketPrice"]["raw"]
                .as_f64()
                .unwrap_or(0.0),
            percent_change: json["quoteSummary"]["result"][0]["price"]["regularMarketChangePercent"]["fmt"]
                .as_str()
                .unwrap_or("err")
                .to_string(),
            amount_change: json["quoteSummary"]["result"][0]["price"]["regularMarketChange"]["fmt"]
                .as_str()
                .unwrap_or("err")
                .to_string(),
            amount_change_raw: json["quoteSummary"]["result"][0]["price"]["regularMarketChange"]["raw"]
                .as_f64()
                .unwrap_or(0.0),
        }
    }

    pub fn get_state(&self) -> QuoteState {
        if self.amount_change_raw > 0.0 {
            QuoteState::POSITIVE
        } else if self.amount_change_raw == 0.0 {
            QuoteState::NEUTRAL
        } else {
            QuoteState::NEGATIVE
        }
    }

    pub fn get_table_headers() -> Vec<String> {
        vec_of_strings!["Ticker", "Price", "% Change", "$ Change"]
    }

    pub fn as_row(&self) -> Vec<String> {
        vec_of_strings![self.ticker, self.price, self.percent_change, self.amount_change]
    }

    pub fn to_chartable(&self) -> (f64, f64) {
        return (
            self.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs() as f64,
            self.price_raw,
        );
    }
}

#[derive(Debug)]
pub struct QuoteError {
    status_code: u16,
    msg: String,
}

impl Error for QuoteError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.status_code, self.msg)
    }
}

impl QuoteError {
    fn new(status_code: u16, msg: &str) -> QuoteError {
        QuoteError {
            status_code: status_code,
            msg: msg.to_string(),
        }
    }
}

pub fn fetch_quote(ticker: &str) -> Result<Quote, QuoteError> {
    let quote_url = format!("https://query1.finance.yahoo.com/v11/finance/quoteSummary/{}", ticker);
    // let quote_url = format!("http://localhost:8080/{}", ticker);
    let resp = ureq::get(&quote_url).query("modules", "summaryDetail,price").call();
    if resp.ok() {
        let json_body = match resp.into_string() {
            Ok(json) => json,
            Err(e) => return Err(QuoteError::new(500, &e.to_string())),
        };
        let json_body: Value = match serde_json::from_str(&json_body) {
            Ok(json) => json,
            Err(e) => return Err(QuoteError::new(500, &e.to_string())),
        };
        Ok(Quote::from_json(&json_body))
    } else {
        Err(QuoteError {
            status_code: resp.status(),
            msg: resp.into_string().unwrap(),
        })
    }
}
