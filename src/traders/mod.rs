mod generic_trader;
pub use generic_trader::{BinanceIndicatorInstance, GenericTrader, StakeSize, TradingFee};

mod macd_trader;
pub use macd_trader::MACDTrader;

mod hodl_trader;
pub use hodl_trader::HODLTrader;

mod dca_trader;
pub use dca_trader::DCATrader;