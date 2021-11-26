use crate::binance_data::BinanceKline;
use crate::traders::{GenericTrader, StakeSize, TradingFee};
use crate::indicators::BinanceIndicatorInstance;
use crate::indicators::DCA;
use yata::core::{Action, IndicatorResult};
use yata::prelude::dd::IndicatorInstanceDyn;
use yata::prelude::*;

use log::debug;


struct IndicatorInstanceWrapper(Box<dyn IndicatorInstanceDyn<BinanceKline>>);
impl BinanceIndicatorInstance for IndicatorInstanceWrapper {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        self.0.next(candle)
    }
}

pub struct DCATrader<'a> {
    trading_fee: TradingFee,
    stake_size: StakeSize,
    kline_feed: &'a mut dyn Iterator<Item = BinanceKline>,
    indicator: Box<dyn BinanceIndicatorInstance>,
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
            indicator: Box::new(dca),
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

    fn indicator(&mut self) -> &mut dyn BinanceIndicatorInstance {
        self.indicator.as_mut()
    }

    fn determine_trade(signals: &[Action]) -> Action {
        debug!("determine trades with hodl signal");
        *signals.get(0).unwrap()
    }
}