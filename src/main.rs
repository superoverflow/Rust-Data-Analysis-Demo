mod account;
mod binance_data;
use account::{Account, Position};
use chrono::NaiveDate;

use binance_data::BinanceKline;

use yata::core::Action;
use yata::indicators::MACD;
use yata::prelude::*;

use env_logger::Env;
use log::info;

pub struct Trader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: &'a mut dyn dd::IndicatorInstanceDyn<BinanceKline>,
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
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        indicator: &'a mut dyn dd::IndicatorInstanceDyn<BinanceKline>,
    ) -> Self {
        Trader {
            trading_fee: TradingFee::PercentageFee(0.5),
            stake_size: StakeSize::FixPercentage(100.0),
            kline_feed,
            indicator,
        }
    }

    pub fn next_trade_session(&mut self, account: &mut Account) -> bool {
        let kline = self.kline_feed.next();
        match kline {
            None => false,
            Some(kline) => {
                let indicator = self.indicator.next(&kline);
                let signals = indicator.signals();
                match signals.get(0).unwrap() {
                    Action::Buy(_) => self.execute_buy(account),
                    Action::Sell(_) => self.execute_sell(account),
                    _ => info!("nothing to do"),
                };
                true
            }
        }
    }
    pub fn execute_buy(&self, _account: &mut Account) {
        info!("Lets buy something");
    }
    pub fn execute_sell(&self, _account: &mut Account) {
        info!("Lets sell something");
    }
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
    let macd = MACD::default();
    let mut macd = macd.init(&first_kline).expect("Unable to initialise MACD");

    let mut klines_iter = klines.into_iter();
    let mut trader = Trader::new(&mut klines_iter, &mut macd);
    while trader.next_trade_session(&mut account) {};
}
