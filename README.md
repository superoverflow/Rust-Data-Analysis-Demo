## Making a small project is the best way to learn
- I am trying to download data from binance and work out how to manipulate the data/analyse the data using rust
- We try to find out some simple trading strategy on some big names in crypto

## Learning Notes
- without `use std::io::prelude::Read;`, `file.read_to_string(&mut contents)` wont work. It need `Read` the read trait
- lots of `unwrap()`, need to think about how to get rid of them
- still not entirely sure why I can pass the String within a function outside the function, I thought it is out of scope
- want to find a more functional way to do stuff
- tokio/async good for fetch files -> thread is too expensive for such simple work
- tokio may have changed the mut ownership so i can pass mut files around from functions to functions?
- reqwest -> status code  -> is_success
- var match -> you may need struct { .. }

## Ideas
- Ideally, we should async pull the zip file, prepare the Kline data asychronously
- We can feed in the Kline to a analytics engine to calculate moving average


## Testing
- integration test - use `python -m http.server` on data folder


## Async
- `block_on` vs `await`
- `futures::join!`