use rdkafka::error::KafkaError;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::GeneratedRecord;

pub struct ProducerManager {
    producer: FutureProducer,
}

impl ProducerManager {
    pub fn new(producer_config: ClientConfig) -> Result<ProducerManager, KafkaError> {
        Ok(ProducerManager {
            producer: producer_config.create()?,
        })
    }

    pub fn start(&mut self, mut rx: Receiver<GeneratedRecord>) -> JoinHandle<(u64, u64)> {
        let producer = self.producer.clone();

        tokio::spawn(async move {
            let mut send_success = 0u64;
            let mut send_fail = 0u64;

            while let Some(gen_rec) = rx.recv().await {
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

            (send_success, send_fail)
        })
    }
}
