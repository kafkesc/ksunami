#[macro_use]
extern crate log;

mod cli;
mod transition;
mod workload;

use std::{thread, time};
use cli::*;
use crate::workload::Workload;

fn main() {
    env_logger::init();

    let cli = Cli::parse_and_validate();
    trace!("CLI input: {:?}", cli);

    let workload = Workload::new(
        cli.min,
        cli.min_sec,
        cli.max,
        cli.max_sec,
        cli.up,
        cli.up_sec,
        cli.down,
        cli.down_sec,
    );
    trace!("Workload created: {:?}", workload);

    info!("");
    info!("Records production will follow this schedule:");
    info!("  1. {} rec/sec for {} seconds", cli.min, cli.min_sec);
    info!("  2. increase in rec/sec along a '{:?}' curve for {} seconds", cli.up, cli.up_sec);
    info!("  3. {} rec/sec for {} seconds", cli.max, cli.max_sec);
    info!("  4. decrease in rec/sec along a '{:?}' curve for {} seconds", cli.down, cli.down_sec);
    info!("  5. repeat from 1.");
    info!("");

    let mut sec = 0u64;
    loop {
        let records_at =workload.records_per_sec_at(sec);
        println!("{sec} sec => {records_at} rec/sec");

        thread::sleep(time::Duration::from_secs(1));
        sec += 1;
    }
}
