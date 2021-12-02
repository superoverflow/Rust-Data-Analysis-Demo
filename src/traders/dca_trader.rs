use crate::data::BinanceKline;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use crate::indicators::BinanceIndicatorInstance;
use crate::indicators::DCA;
use yata::core::{Action};
use yata::prelude::*;

use log::debug;

pub struct DCATrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: Box<dyn BinanceIndicatorInstance>,
}

impl<'a> DCATrader<'a> {
    pub fn new(
        kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
        trading_fee: TradingFee,
    ) -> Self {
        debug!("creating a DCA Trader");
        let dca = DCA::default();
        let dca = dca.init(&kline_feed.next().unwrap()).expect("Unable to initialise DCA");
        Self {
            kline_feed,
            indicator: Box::new(dca),
            trading_fee,
            stake_size: StakeSize::FixAmount(100.0),
        }
    }
}

impl<'a> GenericTrader<'a> for DCATrader<'a> {

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
        debug!("determine trades with dca signal");
        *signals.get(0).unwrap()
    }
}