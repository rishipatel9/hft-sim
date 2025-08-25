use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub px: u64,
    pub qty: u64,
    pub rest: u64,
}
// pub struct Order {
//     pub id: u64,
//     pub side: Side,
//     pub px: u64,
//     pub qty: u64,
//     pub rest: u64,
//     pub timestamp: u128,  // Add this
//     pub order_type: OrderType,  // Add this
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
    IOC,  // Immediate or Cancel
    FOK,  // Fill or Kill
}

pub struct Book {
    next_id: u64,
    buys: BTreeMap<u64, Vec<Order>>,  // highest px first
    sells: BTreeMap<u64, Vec<Order>>, // lowest px first
    log: std::fs::File,
}

impl Book {
    pub fn new() -> Self {
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open("events.log")
            .expect("cannot open log");
        Self {
            next_id: 1,
            buys: BTreeMap::new(),
            sells: BTreeMap::new(),
            log,
        }
    }

    pub fn submit_limit(&mut self, side: Side, px: u64, qty: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let mut order = Order {
            id,
            side,
            px,
            qty,
            rest: qty,
        };

        // Fix: Match against the opposite side without borrowing self mutably twice
        match side {
            Side::Buy => {
                // We're buying, so match against sells (ascending price order)
                self.match_order(&mut order, true);
            }
            Side::Sell => {
                // We're selling, so match against buys (descending price order)  
                self.match_order(&mut order, false);
            }
        }

        if order.rest > 0 {
            match order.side {
                Side::Buy => self.buys.entry(order.px).or_default().push(order.clone()),
                Side::Sell => self.sells.entry(order.px).or_default().push(order.clone()),
            }
            self.log_event("NEW", &order, None);
        }

        id
    }

    fn match_order(&mut self, order: &mut Order, ascending: bool) {
        // Determine which side of the book to match against
        let book_side = match order.side {
            Side::Buy => &mut self.sells,   // Buy orders match against sell orders
            Side::Sell => &mut self.buys,  // Sell orders match against buy orders
        };

        let keys: Vec<u64> = if ascending {
            book_side.keys().cloned().collect() // sells (lowest px first)
        } else {
            book_side.keys().rev().cloned().collect() // buys (highest px first)
        };

        // Collect fill events to log after mutable borrow ends
        let mut fills: Vec<(Order, u64, u64)> = Vec::new();

        for px in keys {
            if order.rest == 0 {
                break;
            }
            
            let should_match = match order.side {
                Side::Buy => order.px >= px,   // Buy at or above the sell price
                Side::Sell => order.px <= px,  // Sell at or below the buy price
            };
            
            if !should_match {
                continue;
            }

            if let Some(orders) = book_side.get_mut(&px) {
                let mut i = 0;
                while i < orders.len() && order.rest > 0 {
                    let resting = &mut orders[i];
                    let fill_qty = order.rest.min(resting.rest);
                    order.rest -= fill_qty;
                    resting.rest -= fill_qty;

                    fills.push((order.clone(), resting.px, fill_qty));

                    if resting.rest == 0 {
                        orders.remove(i);
                        // Don't increment i since we removed an element
                    } else {
                        i += 1;
                    }
                }
                
                // Remove empty price level
                if orders.is_empty() {
                    book_side.remove(&px);
                }
            }
        }

        // Log all fills after mutable borrow ends
        for (order, fill_px, fill_qty) in fills {
            self.log_event("FILL", &order, Some((fill_px, fill_qty)));
        }
    }

    fn log_event(&mut self, event_type: &str, order: &Order, fill: Option<(u64, u64)>) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let (fill_px, fill_qty) = fill.unwrap_or((0, 0));

        let line = format!(
            "{},{},{},{:?},{},{},{},{},{}\n",
            ts,
            event_type,
            order.id,
            order.side,
            order.px,
            order.qty,
            order.rest,
            fill_px,
            fill_qty
        );
        let _ = self.log.write_all(line.as_bytes());
    }
}