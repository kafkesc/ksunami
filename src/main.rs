extern crate core;
#[macro_use]
extern crate log;

use std::error::Error;

use ::rdkafka::ClientConfig;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use cli::*;
use generator::*;
use workload::*;

use crate::producer_sink::ProducerSink;
use crate::records_tap::RecordsTap;

mod cli;
mod generator;
mod logging;
mod producer_sink;
mod rdkafka;
mod records_tap;
mod transition;
mod workload;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = parse_cli_and_init_logging();

    let workload = build_workload(&cli);

    let generator = build_record_generator(&cli)?;

    let producer_config = build_producer_config(&cli);

    let (records_tx, records_rx) = build_records_channel(cli.max as usize);
    let shutdown_rx = setup_shutdown_signal_handler();

    // Create a "tap" of records, based on the workload and generator we just built
    let mut records_tap = RecordsTap::new(workload, generator);

    // Configure a "sink" around a Kafka Producer, based on the producer config we just built
    let mut producer_sink = ProducerSink::new(producer_config)?;

    // Setup channel between "tap" and "sink"
    let records_tap_handle = records_tap.spawn(records_tx, shutdown_rx);
    let producer_sink_handle = producer_sink.spawn(records_rx);

    // Await async tasks: when finished, print out some basic stats
    let sec = records_tap_handle.await?;
    let (success, fail) = producer_sink_handle.await?;
    info!("Records produced for {sec}s: {success} successfully, {fail} failed");

    Ok(())
}

fn parse_cli_and_init_logging() -> Cli {
    // Parse command line input and initialize logging
    let cli = Cli::parse_and_validate();
    logging::init(cli.verbosity_level());

    trace!("Created:\n{:#?}", cli);

    // Log the production that Ksunami intends to do
    info!("");
    info!("Records production will follow this schedule:");
    info!("  1. {} rec/sec for {} seconds", cli.min, cli.min_sec);
    info!("  2. increase in rec/sec along a '{:?}' curve for {} seconds", cli.up, cli.up_sec);
    info!("  3. {} rec/sec for {} seconds", cli.max, cli.max_sec);
    info!("  4. decrease in rec/sec along a '{:?}' curve for {} seconds", cli.down, cli.down_sec);
    info!("  5. repeat from 1.");
    info!("");

    cli
}

fn build_workload(cli: &Cli) -> Workload {
    let workload =
        Workload::new(cli.min, cli.min_sec, cli.max, cli.max_sec, cli.up, cli.up_sec, cli.down, cli.down_sec);

    trace!("Created:\n{:#?}", workload);
    workload
}

fn build_record_generator(cli: &Cli) -> Result<RecordGenerator, std::io::Error> {
    let mut generator = RecordGenerator::new(cli.topic.clone());

    if let Some(k_gen) = &cli.key {
        generator.set_key_generator(k_gen.clone())?;
    }
    if let Some(p_gen) = &cli.payload {
        generator.set_payload_generator(p_gen.clone())?;
    }
    if let Some(part) = cli.partition {
        generator.set_destination_partition(part);
    }
    for kv_pair in &cli.headers {
        generator.add_record_header(kv_pair.0.clone(), kv_pair.1.clone());
    }

    trace!("Created:\n{:#?}", generator);
    Ok(generator)
}

fn build_producer_config(cli: &Cli) -> ClientConfig {
    let mut producer_config = ClientConfig::new();
    producer_config
        .set("bootstrap.servers", cli.bootstrap_brokers.clone())
        .set("client.id", cli.client_id.clone())
        .set("partitioner", cli.partitioner.name());
    for cfg in &cli.config {
        producer_config.set(cfg.0.clone(), cfg.1.clone());
    }

    trace!("Created:\n{:#?}", producer_config);
    producer_config
}

fn build_records_channel(depth: usize) -> (mpsc::Sender<GeneratedRecord>, mpsc::Receiver<GeneratedRecord>) {
    mpsc::channel::<GeneratedRecord>(depth)
}

fn setup_shutdown_signal_handler() -> broadcast::Receiver<()> {
    let (sender, receiver) = broadcast::channel(1);

    // Setup shutdown signal handler:
    // when it's time to shutdown, broadcast to all receiver a unit.
    //
    // NOTE: This handler will be listening on its own dedicated thread.
    if let Err(e) = ctrlc::set_handler(move || {
        info!("Shutting down...");
        sender.send(()).unwrap();
    }) {
        error!("Failed to register signal handler: {e}");
    }

    receiver
}
