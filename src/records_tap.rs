use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time;

use crate::{GeneratedRecord, RecordGenerator, Workload};

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

    pub fn spawn(&mut self, records_tx: mpsc::Sender<GeneratedRecord>, mut shutdown_rx: broadcast::Receiver<()>) -> JoinHandle<u64> {
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
                info!("{sec} sec => {records_at} rec");

                for _ in 0..records_at {
                    match generator.generate_record() {
                        Ok(gen_rec) => {
                            tokio::select! {
                                // Send record to the sink (producer)
                                send_res = records_tx.send(gen_rec) => {
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
