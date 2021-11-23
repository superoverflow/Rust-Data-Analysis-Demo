use crate::account::Account;
use crate::binance_data::BinanceKline;
use crate::indicators::{BinanceIndicatorInstance, HODL, DCA};
use chrono::NaiveDateTime;
use log::{info, debug};
use yata::core::{Action, IndicatorResult};
use yata::prelude::*;
use yata::indicators::MACD;

pub struct IndicatorInstanceWrapper(Box<dyn dd::IndicatorInstanceDyn<BinanceKline>>);
impl BinanceIndicatorInstance for IndicatorInstanceWrapper {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        self.0.next(candle)
    }
}

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
    fn indicator(&mut self) -> &mut IndicatorInstanceWrapper;

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

pub struct MACDTrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: IndicatorInstanceWrapper,
}

impl<'a> GenericTrader<'a> for MACDTrader<'a> {

    fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
        stake_size: StakeSize,
    ) -> Self {
        debug!("creating a MACD Trader");
        let macd = MACD::default();
        let macd = macd.init(&kline_feed.next().unwrap()).expect("Unable to initialise MACD");
        let macd = IndicatorInstanceWrapper(Box::new(macd));
        Self {
            kline_feed,
            indicator: macd,
            trading_fee,
            stake_size,
        }
    }

    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn kline(&mut self) -> &mut dyn Iterator<Item = BinanceKline> {
        self.kline_feed
    }

    fn indicator(&mut self) -> &mut IndicatorInstanceWrapper {
        &mut self.indicator
    }

    fn determine_trade(signals: &[Action]) -> Action {
        debug!("determine trades with macd signal");
        *signals.get(1).unwrap()
    }
}


// HODL Trader
pub struct HODLTrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: IndicatorInstanceWrapper,
}

impl<'a> GenericTrader<'a> for HODLTrader<'a> {
    fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
        _stake_size: StakeSize,
    ) -> Self {
        debug!("creating a HODL Trader");
        let hodl = HODL::default();
        let hodl = hodl.init(&kline_feed.next().unwrap()).expect("Unable to initialise MACD");
        let hodl = IndicatorInstanceWrapper(Box::new(hodl));
        Self {
            kline_feed,
            indicator: hodl,
            trading_fee,
            stake_size: StakeSize::FixPercentage(1.),
        }
    }
    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn kline(&mut self) -> &mut dyn Iterator<Item = BinanceKline> {
        self.kline_feed
    }

    fn indicator(&mut self) -> &mut IndicatorInstanceWrapper {
        &mut self.indicator
    }

    fn determine_trade(signals: &[Action]) -> Action {
        debug!("determine trades with hodl signal");
        *signals.get(0).unwrap()
    }
}


// DCA Trader
pub struct DCATrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: IndicatorInstanceWrapper,
}

impl<'a> GenericTrader<'a> for DCATrader<'a> {
    fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
        _stake_size: StakeSize,
    ) -> Self {
        debug!("creating a DCA Trader");
        let dca = DCA::default();
        let dca = dca.init(&kline_feed.next().unwrap()).expect("Unable to initialise DCA");
        let dca = IndicatorInstanceWrapper(Box::new(dca));
        Self {
            kline_feed,
            indicator: dca,
            trading_fee,
            stake_size: StakeSize::FixAmount(100.0),
        }
    }
    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn kline(&mut self) -> &mut dyn Iterator<Item = BinanceKline> {
        self.kline_feed
    }

    fn indicator(&mut self) -> &mut IndicatorInstanceWrapper {
        &mut self.indicator
    }

    fn determine_trade(signals: &[Action]) -> Action {
        debug!("determine trades with hodl signal");
        *signals.get(0).unwrap()
    }
}