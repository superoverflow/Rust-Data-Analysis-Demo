use crate::binance_data::{BinanceKline, BinanceKlineTrait};
use crate::indicators::BinanceIndicatorInstance;
use chrono::{Datelike, NaiveDateTime};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::prelude::*;
use log::info;

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
        info!("next_binance_kline from DCAInstance");
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