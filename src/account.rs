use chrono::NaiveDateTime;

pub struct Account {
    pub available_fund: f64,
    pub position: Position,
    pub profit_and_loss_history: Vec<TimeValue>,
    pub trade_history: Vec<Trade>,
}

#[derive(Debug, PartialEq)]
pub struct TimeValue {
    timestamp: NaiveDateTime,
    realised_pnl: f64,
    unrealised_pnl: f64,
}

#[derive(Debug, PartialEq)]
pub struct Position {
    pub quantity: f64,
    pub cost: f64,
}

#[derive(Debug, PartialEq)]
pub struct Trade {
    timestamp: NaiveDateTime,
    buy_sell_indicator: BuySellIndicator,
    quantity: f64,
    price: f64,
    fee: f64,
}

#[derive(Debug, PartialEq)]
enum BuySellIndicator {
    Buy,
    Sell,
}

impl Account {
    pub fn new(fund: f64, initial_position: Position, start_timestamp: NaiveDateTime) -> Account {
        let initial_pnl = TimeValue {
            timestamp: start_timestamp,
            realised_pnl: 0.,
            unrealised_pnl: 0.,
        };
        Account {
            available_fund: fund,
            position: initial_position,
            profit_and_loss_history: vec![initial_pnl],
            trade_history: Vec::new(),
        }
    }

    pub fn open(&mut self, timestamp: NaiveDateTime, quantity: f64, price: f64, fee: f64) {
        self.position.cost = (self.position.quantity * self.position.cost + quantity * price)
            / (self.position.quantity + quantity);
        self.position.quantity += quantity;
        self.available_fund -= fee;

        self.trade_history.push(Trade {
            timestamp,
            buy_sell_indicator: BuySellIndicator::Buy,
            quantity,
            price,
            fee,
        });
    }

    pub fn close(&mut self, timestamp: NaiveDateTime, quantity: f64, price: f64, fee: f64) {
        // book profit/loss
        let last_pnl = self.profit_and_loss_history.last().unwrap();
        let realised_pnl = quantity * (price - self.position.cost);
        let current_pnl = TimeValue {
            realised_pnl,
            ..*last_pnl
        };
        self.profit_and_loss_history.push(current_pnl);

        self.position.quantity -= quantity;
        self.available_fund += quantity * price;
        self.available_fund -= fee;

        self.trade_history.push(Trade {
            timestamp,
            buy_sell_indicator: BuySellIndicator::Sell,
            quantity,
            price,
            fee,
        });
    }

    pub fn mark_to_market(&mut self, closing_price: f64, timestamp: NaiveDateTime) {
        let last_pnl = self.profit_and_loss_history.last().unwrap();
        let unrealised_pnl = self.position.quantity * (closing_price - self.position.cost);
        let current_pnl = TimeValue {
            timestamp,
            unrealised_pnl,
            realised_pnl: last_pnl.realised_pnl,
        };
        self.profit_and_loss_history.push(current_pnl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_position() {
        let initial_position = Position {
            quantity: 123.1,
            cost: 10.0,
        };
        let start_timestamp = NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0);
        let account = Account::new(1000.0, initial_position, start_timestamp);
        assert_eq!(account.position.quantity, 123.1);
    }

    #[test]
    fn test_open() {
        let initial_position = Position {
            quantity: 100.0,
            cost: 10.0,
        };
        let start_timestamp = NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0);
        let mut account = Account::new(5000.0, initial_position, start_timestamp);
        let timestamp = NaiveDate::from_ymd(2021, 10, 31).and_hms(0, 0, 0);
        account.open(timestamp, 100.0, 20.0, 0.02);
        assert_eq!(
            account.position,
            Position {
                cost: 15.0,
                quantity: 200.0
            }
        );
        assert_eq!(4999.98, account.available_fund);
        assert_eq!(
            vec![Trade {
                timestamp: NaiveDate::from_ymd(2021, 10, 31).and_hms(0, 0, 0),
                buy_sell_indicator: BuySellIndicator::Buy,
                quantity: 100.0,
                price: 20.0,
                fee: 0.02,
            }],
            account.trade_history
        )
    }

    #[test]
    fn test_close() {
        let initial_position = Position {
            quantity: 100.0,
            cost: 10.0,
        };
        let start_timestamp = NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0);
        let mut account = Account::new(1000.0, initial_position, start_timestamp);
        let timestamp = NaiveDate::from_ymd(2021, 10, 31).and_hms(0, 0, 0);
        account.close(timestamp, 50.0, 20.0, 0.02);
        assert_eq!(
            account.position,
            Position {
                cost: 10.0,
                quantity: 50.0,
            }
        );
        assert_eq!(account.available_fund, 1999.98);
        assert_eq!(
            vec![Trade {
                timestamp: NaiveDate::from_ymd(2021, 10, 31).and_hms(0, 0, 0),
                buy_sell_indicator: BuySellIndicator::Sell,
                quantity: 50.0,
                price: 20.0,
                fee: 0.02,
            }],
            account.trade_history
        )
    }

    #[test]
    fn test_mark_to_market() {
        let initial_position = Position {
            quantity: 100.0,
            cost: 10.0,
        };
        let start_timestamp = NaiveDate::from_ymd(2021, 9, 1).and_hms(0, 0, 0);
        let mut account = Account::new(5000.0, initial_position, start_timestamp);
        let timestamp = NaiveDate::from_ymd(2021, 10, 31).and_hms(0, 0, 0);
        account.mark_to_market(20.0, timestamp.clone());

        let latest_pnl = account.profit_and_loss_history.last().unwrap();
        assert_eq!(
            *latest_pnl,
            TimeValue {
                timestamp,
                realised_pnl: 0.,
                unrealised_pnl: 1000.
            }
        )
    }
}
