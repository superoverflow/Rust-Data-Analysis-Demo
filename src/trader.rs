
use yata::prelude::*;
use chrono:: NaiveDateTime;
use crate::binance_data::BinanceKline;
use crate::account::Account;
use yata::core::Action;
use log::{info, debug};

pub struct Trader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: &'a mut dyn dd::IndicatorInstanceDyn<BinanceKline>
}

#[allow(dead_code)]
pub enum TradingFee {
    FixFee(f64),
    PercentageFee(f64),
}

#[allow(dead_code)]
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