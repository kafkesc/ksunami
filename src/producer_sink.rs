use std::sync::{Arc, Mutex};

use rdkafka::error::KafkaError;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::GeneratedRecord;

/// A "sink" to feed all the records it receives to an [`FutureProducer`].
///
/// It's initialized by passing in the desired configuration of a Producer,
/// and it then feeds every record to said producer, received through the call to [`spawn`].
pub struct ProducerSink {
    producer: FutureProducer,
}

impl ProducerSink {
    /// It takes a `producer_config` [`ClientConfig`], and will create an internal [`FutureProducer`] from it.
    /// If invalid/incomplete, it will return a [`KafkaError`].
    pub fn new(producer_config: ClientConfig) -> Result<ProducerSink, KafkaError> {
        Ok(ProducerSink {
            producer: producer_config.create()?,
        })
    }

    /// "Spawns" the task-loop of the sink, that feeds every record to the producer.
    ///
    /// Every [`GeneratedRecord`] from `records_rx` is converted to [`FutureRecord`] via  [`GeneratedRecord::as_future_record`],
    /// and then sent via the [`FutureProducer`].
    ///
    /// The spawned [`tokio::task`] terminates once the sender side of the given `records_rx` is closed.
    /// On termination, it returns 2 numbers: the number of records successfully sent, and that failed to send.
    pub fn spawn(&mut self, mut records_rx: mpsc::Receiver<GeneratedRecord>) -> JoinHandle<(u64, u64)> {
        let producer = self.producer.clone();

        tokio::spawn(async move {
            let send_success = Arc::new(Mutex::new(0u64));
            let send_fail = Arc::new(Mutex::new(0u64));

            // Stops when `records_rx` receives `None` back:
            // this means that the `records_tx` has been closed (dropped).
            while let Some(gen_rec) = records_rx.recv().await {
                trace!("Generated Record received");

                let producer = producer.clone();
                let send_success = send_success.clone();
                let send_fail = send_fail.clone();

                tokio::spawn(async move {
                    let rec = gen_rec.as_future_record();

                    // Finally, send the record (or wait if the producer internal queue is full)
                    match producer.send(rec, Timeout::Never).await {
                        Ok((partition, offset)) => {
                            *(send_success.lock().unwrap()) += 1;
                            trace!("Delivered => partition: {partition}, offset: {offset}")
                        },
                        Err((e, _)) => {
                            *(send_fail.lock().unwrap()) += 1;
                            error!("Failed record delivery: {:?}", e)
                        },
                    }
                });
            }

            // Return some basic stats:
            // how many did we sent, and how many we failed to send.
            let success = send_success.lock().unwrap();
            let fail = send_fail.lock().unwrap();
            (*success, *fail)
        })
    }
}
