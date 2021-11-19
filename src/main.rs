mod account;
mod binance_data;
use account::{Account, Position};
use chrono::NaiveDate;

use yata::core::{Action, IndicatorResult};
use yata::indicators::MACD;
use yata::prelude::*;

use env_logger::Env;
use log::info;

pub struct Trader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a Vec<binance_data::BinanceKline>,
    indicator: &'a mut dyn dd::IndicatorInstanceDyn<binance_data::BinanceKline>,
}

enum TradingFee {
    FixFee(f64),
    PercentageFee(f64),
}

enum StakeSize {
    FixAmount(f64),
    FixPercentage(f64),
}

impl<'a> Trader<'a> {
    pub fn new(
        kline_feed: &'a Vec<binance_data::BinanceKline>,
        indicator: &'a mut dyn dd::IndicatorInstanceDyn<binance_data::BinanceKline>,
    ) -> Self {
        Trader {
            trading_fee: TradingFee::PercentageFee(0.5),
            stake_size: StakeSize::FixPercentage(100.0),
            kline_feed,
            indicator,
        }
    }

    pub fn next_trade_session(&mut self) {
        self.indicator.next(self.kline_feed.get(0).unwrap());
    }
    pub fn execute_buy(account: &mut Account) {}
    pub fn execute_sell(account: &mut Account) {}
}

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
        let timestamp = kline.end_time;
        let closing_price = kline.close;
        account.mark_to_market(closing_price, timestamp);
        let indicator = macd.next(&kline);

        let first_signal = indicator.signals().first().unwrap();
        let second_signal = indicator.signals().last().unwrap();
        match (first_signal, second_signal) {
            (Action::Buy { .. }, Action::Buy { .. }) => {
                info!(
                    "Buy {:.2}@{:.2}",
                    account.available_fund / closing_price,
                    closing_price
                );
                if account.available_fund > 0.0 {
                    account.open(
                        timestamp,
                        account.available_fund / closing_price,
                        closing_price,
                        account.available_fund * 0.02,
                    )
                };
            }
            (Action::Sell { .. }, _) => {
                info!(
                    "Sell {:.2}@{:.2}",
                    account.position.quantity / closing_price,
                    closing_price
                );
                if account.position.quantity > 0.0 {
                    account.close(
                        timestamp,
                        account.position.quantity,
                        closing_price,
                        account.position.quantity * closing_price * 0.02,
                    )
                };
            }
            (_, Action::Sell { .. }) => {
                info!(
                    "Sell {:.2}@{:.2}",
                    account.position.quantity / closing_price,
                    closing_price
                );
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
        println!(
            "{:?}, {:?}, {:?}",
            account.profit_and_loss_history.last().unwrap(),
            account.position,
            account.available_fund
        );
        indicators.push(indicator);
    }

    info!("calculated into [{}] indicator", indicators.len());
}
