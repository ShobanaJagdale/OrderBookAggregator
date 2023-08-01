use clap::Parser;
use ctrlc;
use std::thread;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use utilities::utils;

pub mod orderbook {
    include!(concat!(env!("OUT_DIR"), "/orderbook.rs"));
}

use orderbook::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use orderbook::{Empty, Level, Summary};

#[derive(Parser, Default, Debug)]
struct Args {
    currency_pair: String,
}

#[derive(Default)]
pub struct Aggregator {}

#[tonic::async_trait]
impl OrderbookAggregator for Aggregator {
    type BookSummaryStream = mpsc::Receiver<Result<Summary, Status>>;
    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);

        let mergedorders = utils::MERGED_ORDER_BOOK.lock().unwrap().return_book();

        let mut bidsvec: Vec<Level> = Vec::new();
        let mut asksvec: Vec<Level> = Vec::new();

        mergedorders.asks.iter().for_each(|ask| {
            asksvec.push(Level {
                exchange: ask.exchange.clone(),
                price: ask.price,
                amount: ask.qty,
            })
        });

        mergedorders.bids.iter().for_each(|bid| {
            bidsvec.push(Level {
                exchange: bid.exchange.clone(),
                price: bid.price,
                amount: bid.qty,
            })
        });

        tokio::spawn(async move {
            let _send = tx
                .send(Ok(Summary {
                    spread: mergedorders.spread,
                    bids: bidsvec.clone(),
                    asks: asksvec.clone(),
                }))
                .await;
        });
        Ok(Response::new(rx))
    }
}

#[tokio::main]
async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let merger = Aggregator::default();
    println!("Streaming server info {}", addr);
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(merger))
        .serve(addr)
        .await?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    let _handler = ctrlc::set_handler(|| {
        println!("Shutting down merger");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let _handle = thread::spawn(|| {
        utils::mergeorderbooks(args.currency_pair);
    });

    let _start = start_server();
}
