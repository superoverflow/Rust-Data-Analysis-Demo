## Making a small project is the best way to learn
- I am trying to download data from binance and work out how to manipulate the data/analyse the data using rust
- We try to find out some simple trading strategy on some big names in crypto

## Goal
- Compare HODL, DCA, SMA, MACD strategies for different coins
- Output Return, Max drawdown, drawdown recovery time

## Assumption
- holding for 1 year
- transaction fee 1.5%

## Learning Notes
- without `use std::io::prelude::Read;`, `file.read_to_string(&mut contents)` wont work. It need `Read` the read trait
- lots of `unwrap()`, need to think about how to get rid of them
- still not entirely sure why I can pass the String within a function outside the function, I thought it is out of scope
- want to find a more functional way to do stuff
- tokio/async good for fetch files -> thread is too expensive for such simple work
- tokio may have changed the mut ownership so i can pass mut files around from functions to functions?
- reqwest -> status code  -> is_success
- var match -> you may need struct { .. }
- mod is quite confusing at first https://www.sheshbabu.com/posts/rust-module-system/
- use `#[cfg(test)]` to do test driven development
- use `env_logger` for logging
- compare `unwrap` vs `expect` vs `?`
- I am trying to make `trader` being able to process any `IndicatorInstance`, i find it hard to write the type due to the `Size` trait constraint. 
  I find you can actually use (Dynamically Size Type(DST))[https://docs.rs/yata/0.4.7/yata/prelude/dd/trait.IndicatorInstanceDyn.html]
- need life time 'a to determine borrow reference for functions


## Ideas
- Ideally, we should async pull the zip file, prepare the Kline data asychronously
- We can feed in the Kline to a analytics engine to calculate moving average
- We can use async stream and set up a channel for sinking the kline data, but for simplicity we choose to do async
- We would like to spawn thread to compare different strategies, while using async to pull data

## Integration Testing
- integration test - use `python -m http.server` on data folder

## Async
- `block_on` vs `await`
- `futures::join!`


