mod dca;
pub use dca::DCA;

mod hodl;
pub use hodl::HODL;


use crate::data::BinanceKline;
use yata::core::IndicatorResult;

pub trait BinanceIndicatorInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult;
}