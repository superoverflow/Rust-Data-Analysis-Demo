mod account;
mod data;
mod indicators;
mod traders;

use account::{Account, Position};
use data::{BinanceKline, get_kline_data};
use chrono::{Duration, NaiveDate, Utc};
use traders::{DCATrader, GenericTrader, HODLTrader, MACDTrader, StakeSize, TradingFee};

use env_logger::Env;
use log::info;

async fn download_kline() -> Vec<BinanceKline> {
    let start_date = NaiveDate::from_ymd(2020, 1, 1);
    let end_date = Utc::today() - Duration::days(1);
    let end_date = end_date.naive_utc();
    let symbol = "ETHUSDT";
    let interval = "1h";
    info!(
        "download data from binance for [{}/{}] from [{}] to [{}]",
        symbol, interval, start_date, end_date
    );
    let klines = get_kline_data(symbol, interval, start_date, end_date).await;
    info!("downloaded [{}] klines", klines.len());
    klines
}

fn initialise_acount(klines: Vec<BinanceKline>) -> Account {
    info!("setting up account");
    let first_kline = klines.first().expect("no klines fetched");
    let start_time = first_kline.start_time;
    let start_fund = 1000.0;
    let start_position = Position {
        quantity: 0.0,
        cost: 0.0,
    };
    let account = Account::new(start_fund, start_position, start_time);
    account
}

#[allow(dead_code)]
fn initialise_macd_trader<'a>(
    klines_iter: &'a mut dyn Iterator<Item = BinanceKline>,
) -> MACDTrader {
    info!("setting up macd trader");
    let stake_size = StakeSize::FixPercentage(1.);
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = MACDTrader::new(klines_iter, trading_fee, stake_size);
    trader
}

#[allow(dead_code)]
fn initialise_hodl_trader<'a>(
    klines_iter: &'a mut dyn Iterator<Item = BinanceKline>,
) -> HODLTrader {
    info!("setting up hodl trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = HODLTrader::new(klines_iter, trading_fee);
    trader
}

#[allow(dead_code)]
fn initialise_dca_trader<'a>(klines_iter: &'a mut dyn Iterator<Item = BinanceKline>) -> DCATrader {
    info!("setting up dca trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = DCATrader::new(klines_iter, trading_fee);
    trader
}

fn loop_kline<'a, T>(trader: &mut T, account: &mut Account)
where
    T: GenericTrader<'a>,
{
    info!("running backtest");
    loop {
        let kline = trader.next_trade_session(account);
        match kline {
            Some(kline) => {
                account.mark_to_market(kline.end_time, kline.close);
            }
            None => break,
        }
    }
}


async fn backtest_macd(klines: Vec<BinanceKline>) -> Account {
    let mut klines_iter = klines.clone().into_iter();
    let mut account = initialise_acount(klines);
    let mut trader = initialise_macd_trader(&mut klines_iter);
    loop_kline(&mut trader, &mut account);
    account
}


async fn backtest_hodl(klines: Vec<BinanceKline>) -> Account {
    let mut klines_iter = klines.clone().into_iter();
    let mut account = initialise_acount(klines);
    let mut trader = initialise_hodl_trader(&mut klines_iter);
    loop_kline(&mut trader, &mut account);
    account
}


async fn backtest_dca(klines: Vec<BinanceKline>) -> Account {
    let mut klines_iter = klines.clone().into_iter();
    let mut account = initialise_acount(klines);
    let mut trader = initialise_dca_trader(&mut klines_iter);
    loop_kline(&mut trader, &mut account);
    account
}

async fn backtest(klines: Vec<BinanceKline>) -> (Account, Account, Account) {
    let macd_account = backtest_macd(klines.clone());
    let hodl_account = backtest_hodl(klines.clone());
    let dca_account = backtest_dca(klines.clone());

    futures::join!(macd_account, hodl_account, dca_account)
}

#[tokio::main]
pub async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let klines = download_kline().await;
    let result = backtest(klines);
    let (macd_account, hodl_account, dca_account) = result.await;

    info!("MACD: {:?}", macd_account.profit_and_loss_history.last().unwrap());
    info!("HODL: {:?}", hodl_account.profit_and_loss_history.last().unwrap());
    info!("DCA : {:?}", dca_account.profit_and_loss_history.last().unwrap());
}
