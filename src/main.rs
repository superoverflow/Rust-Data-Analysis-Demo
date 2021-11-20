mod account;
mod binance_data;
mod trader;
use account::{Account, Position};
use chrono::{Duration, NaiveDate, Utc};
use yata::indicators::MACD;
use yata::prelude::*;

use trader::{StakeSize, TradingFee, MACDTrader, GenericTrader};

use env_logger::Env;
use log::info;

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
    let mut trader = MACDTrader::new(&mut klines_iter, &mut macd, trading_fee, stake_size);

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
