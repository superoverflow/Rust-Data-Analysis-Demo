use crate::binance_data::BinanceKline;
use chrono::{Datelike, NaiveDateTime};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct HODL {}

#[derive(Debug, Clone, Copy)]
pub struct HODLInstance {
    cfg: HODL,
}

impl IndicatorConfig for HODL {
    type Instance = HODLInstance;
    const NAME: &'static str = "HODL";
    fn init<T: OHLCV>(self, _candle: &T) -> Result<Self::Instance, Error> {
        Ok(Self::Instance { cfg: self })
    }
    fn validate(&self) -> bool {
        true
    }
    fn set(&mut self, _name: &str, _value: String) -> Result<(), Error> {
        Ok(())
    }
    fn size(&self) -> (u8, u8) {
        (0, 1)
    }
}

impl Default for HODL {
    fn default() -> Self {
        Self {}
    }
}

impl IndicatorInstance for HODLInstance {
    type Config = HODL;

    fn config(&self) -> &Self::Config {
        &self.cfg
    }

    fn next<T: OHLCV>(&mut self, _candle: &T) -> IndicatorResult {
        IndicatorResult::new(&[], &[Action::Buy(1)])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DCA {}

#[derive(Debug, Clone, Copy)]
pub struct DCAInstance {
    last_timestamp: Option<NaiveDateTime>,
}

pub trait BinanceKlineIndicatorInstance {
    fn next(&mut self, candle: &BinanceKline) -> IndicatorResult;
}

impl DCA {
    fn init(candle: BinanceKline) -> Result<DCAInstance, Error> {
        Ok(DCAInstance {
            last_timestamp: Some(candle.start_time),
        })
    }
}

impl BinanceKlineIndicatorInstance for DCAInstance {
    fn next(&mut self, candle: &BinanceKline) -> IndicatorResult {
        let current_month = candle.start_time.month();
        let last_month = self.last_timestamp.unwrap().month();

        let action = if current_month != last_month {
            Action::Buy(1)
        } else {
            Action::None
        };
        IndicatorResult::new(&[], &[action])
    }
}