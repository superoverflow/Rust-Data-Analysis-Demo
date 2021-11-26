use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use crate::account::Account;
use chrono::NaiveDateTime;
use yata::core::Action;
use log::{info, debug};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TradingFee {
    FixFee(f64),
    PercentageFee(f64),
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum StakeSize {
    FixAmount(f64),
    FixPercentage(f64),
}

pub trait GenericTrader<'a> {
    fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
        stake_size: StakeSize,
    ) -> Self;
    fn determine_trade(signals: &[Action]) -> Action;
    fn stake_size(&self) -> StakeSize;
    fn trading_fee(&self) -> TradingFee;
    fn kline(&mut self) -> &mut dyn Iterator<Item = BinanceKline>;
    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance;

    fn execute_buy(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) {
        let fund = account.available_fund;
        let stake = match self.stake_size() {
            StakeSize::FixAmount(amount) => amount,
            StakeSize::FixPercentage(pct) => fund * pct,
        };
        let fee = match self.trading_fee() {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => stake * pct / (1.0 - pct),
        };
        let quantity = (stake + fee) / price;
        // FIXME: didnt work for DCA Trader
        if quantity > 0.0 {
            info!("B {}, {:.08}, {:.08}, {:.02}", timestamp, quantity, price, stake);
            account.open(timestamp, quantity, price, fee);
        }
    }

    fn execute_sell(&self, timestamp: NaiveDateTime, price: f64, account: &mut Account) {
        let current_position = account.position.quantity;
        let fee = match self.trading_fee() {
            TradingFee::FixFee(fee) => fee,
            TradingFee::PercentageFee(pct) => price * current_position * pct,
        };
        if current_position > 0.0 {
            info!("S {}, {:.08}, {:0.8}", timestamp, current_position, price);
            account.close(timestamp, current_position, price, fee)
        }
    }

    fn next_trade_session(&mut self, account: &mut Account) -> Option<BinanceKline> {
        let kline = self.kline().next();
        match kline {
            None => None,
            Some(kline) => {
                let timestamp = kline.end_time;
                let price = kline.close;

                let indicator = self.indicator().next_binance_kline(&kline);
                let signals = indicator.signals();
                match Self::determine_trade(signals) {
                    Action::Buy(_) => self.execute_buy(timestamp, price, account),
                    Action::Sell(_) => self.execute_sell(timestamp, price, account),
                    _ => debug!("nothing to do"),
                };
                Some(kline)
            }
        }
    }
}