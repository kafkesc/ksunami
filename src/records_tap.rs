use std::time::Duration;

use log::Level::Warn;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;

use crate::{GeneratedRecord, RecordGenerator, Workload};

/// Using a [`Workload`] and a [`RecordGenerator`], it generates records and sends them via a given channel.
///
/// This receives the [`mpsc::Sender`] part of the channel: the [`mpsc::Receiver`] part is
/// assigned to a [`ProducerSink`].
pub struct RecordsTap {
    workload: Workload,
    generator: RecordGenerator,
}

impl RecordsTap {
    pub fn new(workload: Workload, generator: RecordGenerator) -> RecordsTap {
        RecordsTap {
            workload,
            generator,
        }
    }

    /// Instantiates a record-producing loop as async [`tokio::task`].
    ///
    /// Once per second, it queries the internal [`Workload`] for how many records are supposed
    /// to be produced in that instant, and then invokes the internal [`RecordGenerator`] an equal
    /// amount of time. Each record is then sent to the "sink" via the given `records_tx` side of
    /// a channel.
    ///
    /// Additionally, when a `()` is received over the `shutdown_rx` [`broadcast::Receiver`], it
    /// initiates a shutdown: stops producing records and causes the the `records_tx` to be dropped.
    /// This in turn causes the receiver to stop expecting records and shutdown as well.
    pub fn spawn(
        &mut self,
        records_tx: mpsc::Sender<GeneratedRecord>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> JoinHandle<u64> {
        let workload = self.workload.clone();
        let generator = self.generator.clone();

        tokio::spawn(async move {
            // Seconds since we started producing
            let mut sec = 0u64;

            // This is used to set the pace of the records production
            let mut interval = time::interval(time::Duration::from_secs(1));

            let mut shutdown_requested = false;
            while !shutdown_requested {
                // Figure out how many records we need to produce in this second
                let records_at = workload.records_per_sec_at(sec);
                info!("{sec} sec: sending {records_at} recs...");

                for _ in 0..records_at {
                    if log_enabled!(Warn) {
                        // Warn if we have less then 20% capacity on the internal records channel
                        let cap = records_tx.capacity() as f64;
                        let max_cap = records_tx.max_capacity() as f64;
                        let remaining_cap_perc = cap / max_cap;
                        if remaining_cap_perc < 0.2 {
                            warn!(
                                "Remaining capacity of (internal) Records Channel: {:.2}% ({}/{})",
                                remaining_cap_perc * 100f64,
                                cap,
                                max_cap
                            );
                        }
                    }

                    match generator.generate_record() {
                        Ok(gen_rec) => {
                            tokio::select! {
                                // Send record to the sink (producer)
                                send_res = records_tx.send_timeout(gen_rec, Duration::from_millis(10)) => {
                                    if let Err(e) = send_res {
                                        error!("Failed to send record to producer: {e}");
                                    }
                                },

                                // Initiate shutdown: by letting this task conclude,
                                // the "tap" `records_tx` will close, causing the "sink" `records_rx`
                                // to return `None` and conclude its own task.
                                _ = shutdown_rx.recv() => {
                                    info!("Received shutdown signal");
                                    shutdown_requested = true;
                                },
                            }
                        }
                        Err(e) => error!("Failed to generate record: {e}"),
                    }
                }
                info!("{sec} sec: sent {records_at} recs");

                // Await next cycle: we do the awaiting at this stage, so that we can start producing
                // for this second as soon as possible, instead of using some of that time to produce the
                // records.
                interval.tick().await;
                sec += 1;
            }

            // Return for how many seconds has this been producing records
            sec
        })
    }
}
