mod binance_data;

mod account;
use account::{Account, Position};
use chrono::NaiveDate;

use yata::core::{Action, IndicatorResult};
use yata::indicators::MACD;
use yata::prelude::*;

use env_logger::Env;
use log::info;

#[tokio::main]
pub async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("download data from binance");
    let klines = binance_data::get_kline_data(
        String::from("ETHUSDT"),
        String::from("1d"),
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2021, 11, 21),
    )
    .await;
    info!("downloaded [{}] klines", klines.len());

    let first_kline = klines.first().expect("no klines fetched");
    let start_time = first_kline.start_time;
    let start_fund = 1000.0;
    let start_position = Position {
        quantity: 0.0,
        cost: 0.0,
    };
    let mut account = Account::new(start_fund, start_position, start_time);
    let mut indicators: Vec<IndicatorResult> = Vec::new();

    let macd = MACD::default();
    let mut macd = macd.init(&first_kline).expect("Unable to initialise MACD");
    for kline in klines {
        let timestamp = kline.start_time;
        let closing_price = kline.close;
        account.mark_to_market(closing_price, timestamp);
        let indicator = macd.next(&kline);
        let first_signal = indicator.signals().first().unwrap();
        let second_signal = indicator.signals().last().unwrap();

        match (first_signal, second_signal) {
            (Action::Buy { .. }, Action::Buy { .. }) => {
                if account.available_fund > 0.0 {
                    account.open(
                        timestamp,
                        account.available_fund / closing_price,
                        closing_price,
                        account.available_fund * 0.02,
                    )
                }
            }
            (Action::Sell { .. }, _) => {
                if account.position.quantity > 0.0 {
                    account.close(
                        timestamp,
                        account.position.quantity,
                        closing_price,
                        account.position.quantity * closing_price * 0.02,
                    )
                }
            }
            (_, Action::Sell { .. }) => {
                if account.position.quantity > 0.0 {
                    account.close(
                        timestamp,
                        account.position.quantity,
                        closing_price,
                        account.position.quantity * closing_price * 0.02,
                    )
                }
            }
            _ => (),
        }
        println!("{:?}", account.profit_and_loss_history.last().unwrap());
        indicators.push(indicator);
    }

    info!("calculated into [{}] indicator", indicators.len());
}
