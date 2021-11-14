mod binance_data;
use chrono::NaiveDate;

use yata::core::IndicatorResult;
use yata::indicators::MACD;
use yata::prelude::*;

// ---- backtest ----
// fn available_fund
// fn current_position

// --- end result ---
// fn kline
// fn indicators
// fn signals
// fn profit_and_loss

#[tokio::main]
pub async fn main() {
    println!("download data");
    let data = binance_data::get_kline_data(
        String::from("ETHUSDT"),
        String::from("4h"),
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2021, 11, 21),
    )
    .await;
    println!("downloaded {}", data.len());
    let macd = MACD::default();
    let mut macd = macd
        .init(&data.first().unwrap())
        .expect("Unable to initialise MACD");

    let mut result: Vec<IndicatorResult> = Vec::new();
    for candle in data {
        let indicator = macd.next(&candle);
        result.push(indicator);
        println!("{:#?}", indicator);
    }
    println!("calculated into [{}] indicator", result.len());
}
