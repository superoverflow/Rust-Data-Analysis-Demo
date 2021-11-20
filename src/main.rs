mod account;
mod binance_data;
use account::{Account, Position};
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};

use binance_data::BinanceKline;

use yata::core::Action;
use yata::indicators::MACD;
use yata::prelude::*;

use env_logger::Env;
use log::{debug, info};

pub struct Trader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: &'a mut dyn dd::IndicatorInstanceDyn<BinanceKline>
}

pub enum TradingFee {
    FixFee(f64),
    PercentageFee(f64),
}

pub enum StakeSize {
    FixAmount(f64),
    FixPercentage(f64),
}

impl<'a> Trader<'a> {
    pub fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        indicator: &'a mut dyn dd::IndicatorInstanceDyn<BinanceKline>,
        trading_fee: TradingFee,
        stake_size: StakeSize
    ) -> Self {
        Trader {
            kline_feed,
            indicator,
            trading_fee: trading_fee,
            stake_size: stake_size
        }
    }

    pub fn next_trade_session(&mut self, account: &mut Account) -> Option<BinanceKline> {
        let kline = self.kline_feed.next();
        match kline {
            None => None,
            Some(kline) => {
                let timestamp = kline.end_time;
                let price = kline.close;

                let indicator = self.indicator.next(&kline);
                let signals = indicator.signals();
                match signals.get(1).unwrap() {
                    Action::Buy(_) => self.execute_buy(timestamp, price, account),
                    Action::Sell(_) => self.execute_sell(timestamp, price, account),
                    _ => debug!("nothing to do"),
                };
                Some(kline)
            }
        }
    }

    pub fn execute_buy(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) {
        let fund = account.available_fund;
        let stake = match self.stake_size {
            StakeSize::FixAmount(amount) => amount,
            StakeSize::FixPercentage(pct) => fund * pct,
        };
        let fee = match self.trading_fee {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => stake * pct / (1.0 - pct),
        };
        let quantity = (stake + fee) / price;

        if quantity > 0.0 {
            info!("B {}, {:.08}, {:.08}", timestamp, quantity, price);
            account.open(timestamp, quantity, price, fee);
        }
    }

    pub fn execute_sell(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) {
        let current_position = account.position.quantity;
        let fee = match self.trading_fee {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => price * current_position * pct,
        };
        if current_position > 0.0 {
            info!("S {}, {:.08}, {:0.8}", timestamp, current_position, price);
            account.close(timestamp, current_position, price, fee)
        }
    }
}

#[tokio::main]
pub async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let start_date = NaiveDate::from_ymd(2020, 1, 1);
    let end_date = Utc::today() - Duration::days(1);
    let end_date = end_date.naive_utc();
    let symbol = "BTCUSDT";
    let interval = "1d";
    info!("download data from binance for [{}/{}] from [{}] to [{}]", symbol, interval, start_date, end_date);
    let klines = binance_data::get_kline_data(symbol, interval, start_date, end_date).await;
    info!("downloaded [{}] klines", klines.len());

    info!("setting up indicator");
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

    info!("setting up trader");
    let stake_size = StakeSize::FixPercentage(1.);
    let trading_fee = TradingFee::PercentageFee(0.5);
    let mut klines_iter = klines.into_iter();
    let mut trader = Trader::new(&mut klines_iter, &mut macd, trading_fee, stake_size);

    info!("running backtest");
    loop {
        let kline = trader.next_trade_session(&mut account);
        match kline {
            Some(kline) => {
                account.mark_to_market(kline.end_time, kline.close);
                info!("{:?}", account.profit_and_loss_history.last().unwrap());
            }
            None => break,
        }
    }
}
