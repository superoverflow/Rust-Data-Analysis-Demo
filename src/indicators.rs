use crate::binance_data::{BinanceKline, BinanceKlineTrait};
use chrono::{Datelike, NaiveDateTime};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct HODL {}

#[derive(Debug, Clone, Copy)]
pub struct HODLInstance {
    cfg: HODL,
}

pub trait BinanceIndicatorInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult;
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

impl BinanceIndicatorInstance for HODLInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        self.next(candle)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct DCA {}

#[derive(Debug, Clone, Copy)]
pub struct DCAInstance {
    cfg: DCA,
    last_timestamp: Option<NaiveDateTime>,
}

impl IndicatorConfig for DCA {
    type Instance = DCAInstance;
    const NAME: &'static str = "DCA";
    fn init<T: OHLCV>(self, _candle: &T) -> Result<Self::Instance, Error> {
        Ok(Self::Instance {
            last_timestamp: None,
            cfg: self,
        })
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

impl Default for DCA {
    fn default() -> Self {
        Self {}
    }
}



impl IndicatorInstance for DCAInstance {
    type Config = DCA;

    fn config(&self) -> &Self::Config {
        &self.cfg
    }

    fn next<T: OHLCV>(&mut self, _candle: &T) -> IndicatorResult {
        IndicatorResult::new(&[], &[Action::Buy(1)])
    }
}

impl BinanceIndicatorInstance for DCAInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        let current_month = (*candle).start_time().month();
        let last_month = self.last_timestamp.unwrap().month();

        let action = if current_month != last_month {
            Action::Buy(1)
        } else {
            Action::None
        };
        IndicatorResult::new(&[], &[action])
    }
}
