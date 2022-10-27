use rdkafka::error::KafkaError;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::GeneratedRecord;

pub struct ProducerSink {
    producer: FutureProducer,
}

impl ProducerSink {
    pub fn new(producer_config: ClientConfig) -> Result<ProducerSink, KafkaError> {
        Ok(ProducerSink {
            producer: producer_config.create()?,
        })
    }

    pub fn spawn(&mut self, mut records_rx: mpsc::Receiver<GeneratedRecord>) -> JoinHandle<(u64, u64)> {
        let producer = self.producer.clone();

        tokio::spawn(async move {
            let mut send_success = 0u64;
            let mut send_fail = 0u64;

            // Stops when `records_rx` receives `None` back:
            // this means that the `records_tx` has been closed (dropped).
            while let Some(gen_rec) = records_rx.recv().await {
                trace!("Generated Record received");

                let rec = gen_rec.as_future_record();

                // Finally, send the record
                match producer.send(rec, Timeout::Never).await {
                    Ok((partition, offset)) => {
                        send_success += 1;
                        trace!("Delivered => partition: {partition}, offset: {offset}")
                    }
                    Err((e, _)) => {
                        send_fail += 1;
                        error!("Failed record delivery: {:?}", e)
                    }
                }
            }

            // Return some basic stats:
            // how many did we sent, and how many we failed to send.
            (send_success, send_fail)
        })
    }
}
