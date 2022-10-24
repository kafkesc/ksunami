extern crate core;
#[macro_use]
extern crate log;

use tokio::time;

use cli::*;

use crate::workload::Workload;

mod cli;
mod generator;
mod logging;
mod transition;
mod workload;

#[tokio::main]
async fn main() {
    // Parse command line input and initialize logging
    let cli = Cli::parse_and_validate();
    logging::init(cli.verbose);
    trace!("CLI input: {:?}", cli);

    // Configure workload
    let workload =
        Workload::new(cli.min, cli.min_sec, cli.max, cli.max_sec, cli.up, cli.up_sec, cli.down, cli.down_sec);

    // Log the production that Ksunami intends to do
    info!("");
    info!("Records production will follow this schedule:");
    info!("  1. {} rec/sec for {} seconds", cli.min, cli.min_sec);
    info!("  2. increase in rec/sec along a '{:?}' curve for {} seconds", cli.up, cli.up_sec);
    info!("  3. {} rec/sec for {} seconds", cli.max, cli.max_sec);
    info!("  4. decrease in rec/sec along a '{:?}' curve for {} seconds", cli.down, cli.down_sec);
    info!("  5. repeat from 1.");
    info!("");

    // // let bootstrap_servers = "";
    // // let topic = "";
    // //
    // // Create the `FutureProducer` to produce asynchronously.
    // let producer: FutureProducer = ClientConfig::new()
    //     .set("bootstrap.servers", bootstrap_servers)
    //     .set("message.timeout.ms", "5000")
    //     .create()
    //     .expect("Producer creation error");
    // //
    // let produce_future =
    //     producer.send(FutureRecord::to(topic).key("some key").payload(""), Timeout::Never);

    let mut sec = 0u64;
    let mut interval = time::interval(time::Duration::from_secs(1));
    loop {
        // Figure out how many records we need to produce in this second
        let records_at = workload.records_per_sec_at(sec);
        debug!("At {sec}s will produce {records_at}recs");

        // TODO Prepare records: iterate over a generator of some kind?
        // TODO Accumulate calls to `producer.send()`, resulting in a collection of Futures

        // Await next cycle: we do the awaiting at this stage, so that we can start producing
        // for this second as soon as possible, instead of using some of that time to produce the
        // records.
        interval.tick().await;

        // TODO Produce records
        info!("{sec} sec => {records_at} rec");
        sec += 1;
    }
}
