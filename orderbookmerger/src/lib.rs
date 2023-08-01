pub mod utils {
    use ordered_float::OrderedFloat;
    use serde_json::json;
    use std::sync::Mutex;
    use tungstenite::{connect, Message};
    use url::Url;

    pub mod messages {

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        pub struct Priceinfo {
            pub price: String,
            pub qty: String,
        }
        #[derive(serde::Deserialize, Debug)]
        pub struct Data {
            pub bids: Vec<Priceinfo>,
            pub asks: Vec<Priceinfo>,
        }
        #[derive(serde::Deserialize, Debug)]
        pub struct BitstampMessage {
            pub data: Data,
        }
        #[derive(serde::Deserialize, Debug)]
        pub struct BinanceMessage {
            pub bids: Vec<Priceinfo>,
            pub asks: Vec<Priceinfo>,
        }
        #[derive(Debug, Clone)]
        pub struct Order {
            pub exchange: String,
            pub price: f64,
            pub qty: f64,
        }
    }
    pub struct OrderBook {
        pub bids: Vec<messages::Order>,
        pub asks: Vec<messages::Order>,
        pub spread: f64,
    }

    impl OrderBook {
        fn update_book(&mut self, bids: Vec<messages::Order>, asks: Vec<messages::Order>) {
            self.bids.clear();
            self.asks.clear();
            self.bids = bids;
            self.asks = asks;
            self.spread = (self.asks[0].price - self.bids[0].price).abs();
        }

        pub fn return_book(&mut self) -> OrderBook {
            return OrderBook {
                bids: self.bids.clone(),
                asks: self.asks.clone(),
                spread: self.spread,
            };
        }
    }

    pub const EXCHANGE1: &str = "bitstamp";
    pub const EXCHANGE2: &str = "binance";
    pub static MERGED_ORDER_BOOK: Mutex<OrderBook> = Mutex::new(OrderBook {
        bids: Vec::new(),
        asks: Vec::new(),
        spread: 0.0,
    });

    pub fn mergebids(
        bids1: Vec<messages::Priceinfo>,
        bids2: Vec<messages::Priceinfo>,
    ) -> Vec<messages::Order> {
        let mut i = 0;
        let mut j = 0;
        let mut mergedbids: Vec<messages::Order> = Vec::new();
        while mergedbids.len() < 11 {
            let price1 = bids1[i].price.parse::<OrderedFloat<f64>>().unwrap();
            let price2 = bids2[j].price.parse::<OrderedFloat<f64>>().unwrap();
            if price1 >= price2 {
                mergedbids.push(messages::Order {
                    price: bids1[i].price.parse::<f64>().unwrap(),
                    exchange: EXCHANGE1.to_string(),
                    qty: bids1[i].qty.parse::<f64>().unwrap(),
                });
                i += 1;
            } else {
                mergedbids.push(messages::Order {
                    price: bids2[j].price.parse::<f64>().unwrap(),
                    exchange: EXCHANGE2.to_string(),
                    qty: bids2[j].qty.parse::<f64>().unwrap(),
                });
                j += 1;
            }
        }

        return mergedbids;
    }

    pub fn mergeasks(
        asks1: Vec<messages::Priceinfo>,
        asks2: Vec<messages::Priceinfo>,
    ) -> Vec<messages::Order> {
        let mut i = 0;
        let mut j = 0;
        let mut mergedasks: Vec<messages::Order> = Vec::new();
        while mergedasks.len() < 11 {
            let price1 = asks1[i].price.parse::<OrderedFloat<f64>>().unwrap();
            let price2 = asks2[j].price.parse::<OrderedFloat<f64>>().unwrap();
            if price1 <= price2 {
                mergedasks.push(messages::Order {
                    exchange: EXCHANGE1.to_string(),
                    price: asks1[i].price.parse::<f64>().unwrap(),
                    qty: asks1[i].qty.parse::<f64>().unwrap(),
                });
                i += 1;
            } else {
                mergedasks.push(messages::Order {
                    price: asks2[j].price.parse::<f64>().unwrap(),
                    exchange: EXCHANGE2.to_string(),
                    qty: asks2[j].qty.parse::<f64>().unwrap(),
                });
                j += 1;
            }
        }
        return mergedasks;
    }

    pub fn mergeorderbooks(currency_pair: String) {
        let (mut socket1, _response) =
            connect(Url::parse("wss://ws.bitstamp.net").unwrap()).expect("Can't connect");
        println!("Connected to Bitstamp");
        let symbol = "order_book_".to_owned() + &currency_pair;
        socket1
            .write_message(
                Message::Text(
                    json!({
                        "event": "bts:subscribe",
                        "data": {"channel": symbol }
                    })
                    .to_string(),
                )
                .into(),
            )
            .expect("Error sending message");

        let _response = socket1.read_message().expect("Error reading message");

        let urlstr =
            "wss://stream.binance.com:9443/ws/".to_owned() + &currency_pair + "@depth20@100ms";

        let (mut socket2, _response) =
            connect(Url::parse(&urlstr).unwrap()).expect("Can't connect");
        println!("Connected to Binance");
        let param1 = currency_pair.clone() + "@aggTrade";
        let param2 = currency_pair.clone() + "@depth";
        socket2
            .write_message(
                Message::Text(
                    json!({
                     "method": "SUBSCRIBE",
                     "params": [
                        param1,
                        param2
                        ],"id": 1
                    })
                    .to_string(),
                )
                .into(),
            )
            .expect("Error sending message");
        println!("Processing websocket feeds");
        loop {
            let response1 = socket1.read_message().expect("Error reading message");
            let result1: Result<messages::BitstampMessage, serde_json::Error> =
                serde_json::from_str(response1.to_text().unwrap());
            let _value = match result1 {
                Ok(ref _response1) => {}
                Err(_) => {
                    continue;
                }
            };
            let response2 = socket2.read_message().expect("Error reading message");
            let result2: Result<messages::BinanceMessage, serde_json::Error> =
                serde_json::from_str(response2.to_text().unwrap());
            let _value = match result2 {
                Ok(ref _response2) => {}
                Err(_) => {
                    continue;
                }
            };
            let book1 = result1.unwrap();
            let book2 = result2.unwrap();
            let mergedbids = mergebids(book1.data.bids, book2.bids);
            let mergedasks = mergeasks(book1.data.asks, book2.asks);

            MERGED_ORDER_BOOK
                .lock()
                .unwrap()
                .update_book(mergedbids, mergedasks);
        }
    }
}
