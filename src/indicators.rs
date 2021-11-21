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
