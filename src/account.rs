use chrono::NaiveDateTime;

struct Account {
    available_fund: f64,
    position: Position,
    profit_and_loss_history: Vec<TimeValue>,
    trade_history: Vec<Trade>,
}

#[derive(Debug)]
struct TimeValue {
    time_stamp: NaiveDateTime,
    value: f64,
}

#[derive(Debug, PartialEq)]
struct Position {
    quantity: f64,
    cost: f64,
}

struct Trade {
    trade_time: NaiveDateTime,
    buy_sell_indicator: BuySellIndicator,
    quantity: f64,
    price: f64,
}

enum BuySellIndicator {
    Buy,
    Sell,
}

impl Account {
    fn new(fund: f64, initial_position: Position) -> Account {
        Account {
            available_fund: fund,
            position: initial_position,
            profit_and_loss_history: Vec::new(),
            trade_history: Vec::new(),
        }
    }

    fn open(&mut self, quantity: f64, price: f64, fee: f64) {
        self.position.cost = (self.position.quantity * self.position.cost + quantity * price)
            / (self.position.quantity + quantity);
        self.position.quantity += quantity;
        self.available_fund -= fee;
    }

    fn close(&mut self, quantity: f64, price: f64, fee: f64) {
        self.position.quantity -= quantity;
        self.available_fund += quantity * price;
        self.available_fund -= fee;
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
        let account = Account::new(1000.0, initial_position);
        assert_eq!(account.position.quantity, 123.1);
    }

    #[test]
    fn test_open() {
        let initial_position = Position {
            quantity: 100.0,
            cost: 10.0,
        };
        let mut account = Account::new(5000.0, initial_position);
        account.open(100.0, 20.0, 0.02);
        assert_eq!(
            account.position,
            Position {
                cost: 15.0,
                quantity: 200.0
            }
        );
        assert_eq!(4999.98, account.available_fund)
    }

    #[test]
    fn test_close() {
        let initial_position = Position {
            quantity: 100.0,
            cost: 10.0,
        };
        let mut account = Account::new(1000.0, initial_position);
        account.close(50.0, 20.0, 0.02);
        assert_eq!(
            account.position,
            Position {
                cost: 10.0,
                quantity: 50.0,
            }
        );
        assert_eq!(account.available_fund, 1999.98);
    }
}
