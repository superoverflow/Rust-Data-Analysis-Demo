mod binance_data;
mod account;
use chrono::NaiveDate;

use yata::core::IndicatorResult;
use yata::indicators::MACD;
use yata::prelude::*;

use env_logger::Env;
use log::info;

// --- end result ---
// fn kline
// fn indicators
// fn signals
// fn profit_and_loss

#[tokio::main]
pub async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("download data from binance");
    let data = binance_data::get_kline_data(
        String::from("ETHUSDT"),
        String::from("4h"),
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2021, 11, 21),
    )
    .await;
    info!("downloaded [{}] klines", data.len());


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
    info!("calculated into [{}] indicator", result.len());
}
