extern crate core;
#[macro_use]
extern crate log;

use std::error::Error;

use ::rdkafka::ClientConfig;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time;

use cli::*;
use generator::*;
use workload::*;

use crate::producer_manager::ProducerManager;

mod cli;
mod generator;
mod logging;
mod producer_manager;
mod rdkafka;
mod transition;
mod workload;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line input and initialize logging
    let cli = Cli::parse_and_validate();
    logging::init(cli.verbosity_level());
    trace!("Created:\n{:#?}", cli);

    // Configure workload
    let workload =
        Workload::new(cli.min, cli.min_sec, cli.max, cli.max_sec, cli.up, cli.up_sec, cli.down, cli.down_sec);
    trace!("Created:\n{:#?}", workload);

    // Log the production that Ksunami intends to do
    info!("");
    info!("Records production will follow this schedule:");
    info!("  1. {} rec/sec for {} seconds", cli.min, cli.min_sec);
    info!("  2. increase in rec/sec along a '{:?}' curve for {} seconds", cli.up, cli.up_sec);
    info!("  3. {} rec/sec for {} seconds", cli.max, cli.max_sec);
    info!("  4. decrease in rec/sec along a '{:?}' curve for {} seconds", cli.down, cli.down_sec);
    info!("  5. repeat from 1.");
    info!("");

    // Configure record generator from CLI input
    let mut generator = RecordGenerator::new(cli.topic);
    if let Some(k_gen) = cli.key {
        generator.set_key_generator(k_gen)?;
    }
    if let Some(p_gen) = cli.payload {
        generator.set_payload_generator(p_gen)?;
    }
    if let Some(part) = cli.partition {
        generator.set_destination_partition(part);
    }
    for kv_pair in cli.headers {
        generator.add_record_header(kv_pair.0, kv_pair.1);
    }
    trace!("Created:\n{:#?}", generator);

    // Configure Kafka producer
    let mut producer_config = ClientConfig::new();
    producer_config
        .set("bootstrap.servers", cli.bootstrap_brokers)
        .set("client.id", cli.client_id)
        .set("partitioner", cli.partitioner.name());
    for cfg in cli.config {
        producer_config.set(cfg.0, cfg.1);
    }
    trace!("Created:\n{:#?}", producer_config);

    // Configure the producer manager
    let mut prod_man = ProducerManager::new(producer_config)?;

    // Setup channel between generator and producer
    let (tx, rx) = create_generated_records_channel(cli.max as usize);

    // Setup the generator manager
    let generator_manager_handle = tokio::spawn(async move {
        // Seconds since we started producing
        let mut sec = 0u64;

        // This is used to set the pace of the records production
        let mut interval = time::interval(time::Duration::from_secs(1));

        loop {
            // Figure out how many records we need to produce in this second
            let records_at = workload.records_per_sec_at(sec);
            info!("{sec} sec => {records_at} rec");

            for _ in 0..records_at {
                match generator.generate_record() {
                    Ok(gen_rec) => {
                        if let Err(e) = tx.send(gen_rec).await {
                            error!("Failed to send record to producer: {e}");
                        }
                    }
                    Err(e) => error!("Failed to generate record: {e}"),
                }
            }

            // Await next cycle: we do the awaiting at this stage, so that we can start producing
            // for this second as soon as possible, instead of using some of that time to produce the
            // records.
            interval.tick().await;
            sec += 1;
        }
    });

    let prod_man_handle = prod_man.start(rx);
    generator_manager_handle.await.unwrap();
    let (success, fail) = prod_man_handle.await.unwrap();

    info!("Records sent: success={success}, fail={fail}");
    Ok(())
}

fn create_generated_records_channel(depth: usize) -> (Sender<GeneratedRecord>, Receiver<GeneratedRecord>) {
    mpsc::channel::<GeneratedRecord>(depth)
}
