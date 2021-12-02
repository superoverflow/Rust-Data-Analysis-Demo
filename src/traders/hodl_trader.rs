use crate::data::BinanceKline;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use crate::indicators::BinanceIndicatorInstance;
use crate::indicators::HODL;
use yata::core::Action;
use yata::prelude::*;

use log::debug;


pub struct HODLTrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: Box<dyn BinanceIndicatorInstance>,
}

impl<'a> HODLTrader<'a> {
    pub fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
    ) -> Self {
        debug!("creating a HODL Trader");
        let hodl = HODL::default();
        let hodl = hodl.init(&kline_feed.next().unwrap()).expect("Unable to initialise MACD");
        let hodl = Box::new(hodl);
        Self {
            kline_feed,
            indicator: hodl,
            trading_fee,
            stake_size: StakeSize::FixPercentage(1.),
        }
    }
}

impl<'a> GenericTrader<'a> for HODLTrader<'a> {
    fn stake_size(&self) -> StakeSize {
        self.stake_size
    }

    fn trading_fee(&self) -> TradingFee {
        self.trading_fee
    }

    fn kline(&mut self) -> &mut dyn Iterator<Item = BinanceKline> {
        self.kline_feed
    }

    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance {
        self.indicator.as_mut()
    }

    fn determine_trade(signals: &[Action]) -> Action {
        debug!("determine trades with hodl signal");
        *signals.get(0).unwrap()
    }
}