use std::{thread, time::Duration};
pub mod orderbook {
    include!(concat!(env!("OUT_DIR"), "/orderbook.rs"));
}

use ctrlc;
use orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use orderbook::Empty;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect()
        .await?;
    let mut client = OrderbookAggregatorClient::new(channel);
    let _handler = ctrlc::set_handler(|| {
        println!("Shutting down");
        std::process::exit(0);
    });

    loop {
        let request = tonic::Request::new(Empty {});
        let mut response = client.book_summary(request).await?.into_inner();
        while let Some(res) = response.message().await? {
            clearscreen::clear().expect("Error clearing screen");

            if res.asks.is_empty() && res.bids.is_empty() {
                println!("No data available. Please check the symbols, may be invalid!!");
            } else {
                println!();
                println!("spread = {:.6}", res.spread);

                println!("asks [ ");
                res.asks.iter().for_each(|ask| {
                    println!(
                        "exchange : {:?} , price : {:?} , amount : {:?}",
                        ask.exchange, ask.price, ask.amount
                    )
                });
                println!(" ] ");

                println!("bids [ ");
                res.bids.iter().for_each(|bid| {
                    println!(
                        "exchange : {:?} , price : {:?} , amount: {:?}",
                        bid.exchange, bid.price, bid.amount
                    )
                });
                println!(" ] ");
            }
        }
     
    }
    #[allow(unreachable_code)]
    Ok(())
}
