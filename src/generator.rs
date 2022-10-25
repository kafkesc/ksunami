use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;
use std::str::from_utf8;

use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};

/// Helps to generate a possible value used in [`RecordGenerator`].
///
/// Specifically, this is used for the [`RecordGenerator::key_field`] and [`RecordGenerator::payload_field`],
/// to specify what content should be generated for those fields when a Kafka record is generated
/// (via [`RecordGenerator::generate_record`]).
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueGenerator {
    /// A user provided string.
    Fixed(String),

    /// The content of a file.
    File(PathBuf),

    /// A random alphanumeric string.
    RandAlphaNum(usize),

    /// A random bytes array.
    RandBytes(usize),

    /// A random (signed) integer.
    RandInt(i64, i64),

    /// A random float.
    RandFloat(f64, f64),
}

impl ValueGenerator {
    /// Generates a `Vec<u8>` of bytes containing the value created by this generator, or an error.
    fn generate(&self) -> Result<Vec<u8>, Error> {
        match self {
            ValueGenerator::Fixed(s) => Ok(s.as_bytes().to_vec()),
            ValueGenerator::File(bp) => {
                debug!("Loading content of file {:?}", bp);
                let mut f = File::open(bp)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;

                Ok(buf)
            }
            ValueGenerator::RandAlphaNum(l) => {
                let rand_alpha = Alphanumeric.sample_string(&mut thread_rng(), *l);

                Ok(rand_alpha.as_bytes().to_vec())
            }
            ValueGenerator::RandBytes(l) => {
                let random_bytes: Vec<u8> = (0..*l).map(|_| thread_rng().gen::<u8>()).collect();

                Ok(random_bytes)
            }
            ValueGenerator::RandInt(min, max) => {
                let random_int = thread_rng().gen_range(*min..=*max);

                Ok(random_int.to_be_bytes().to_vec())
            }
            ValueGenerator::RandFloat(min, max) => {
                let random_float = thread_rng().gen_range(*min..=*max);

                Ok(random_float.to_be_bytes().to_vec())
            }
        }
    }

    /// Implementation of [`clap::value_parser`], used to create an argument by parsing a user-provided value.
    pub fn clap_value_parser(gen_field: &str) -> Result<ValueGenerator, String> {
        let (gen_field_type, gen_field_content) = match gen_field.split_once(':') {
            None => {
                return Err("Should have 'TYPE:CONTENT' format".to_string());
            }
            Some((t, c)) => (t, c),
        };

        match gen_field_type {
            "string" => Ok(ValueGenerator::Fixed(gen_field_content.to_string())),
            "file" => {
                let path = PathBuf::from(gen_field_content);

                if !path.exists() {
                    Err(format!("File '{}' does not exist", path.display()))
                } else {
                    Ok(ValueGenerator::File(path))
                }
            }
            "alpha" => {
                let res = gen_field_content.parse::<usize>();

                match res {
                    Err(e) => Err(format!("Failed to parse 'SIZE' from 'alpha:SIZE': {e}")),
                    Ok(size) => Ok(ValueGenerator::RandAlphaNum(size)),
                }
            }
            "bytes" => {
                let res = gen_field_content.parse::<usize>();

                match res {
                    Err(e) => Err(format!("Failed to parse 'SIZE' from 'bytes:SIZE': {e}")),
                    Ok(size) => Ok(ValueGenerator::RandBytes(size)),
                }
            }
            "int" => match gen_field_content.split_once('-') {
                None => Err("Inclusive range should have 'min-max' format".to_string()),
                Some((min_str, max_str)) => {
                    let min = match min_str.parse::<i64>() {
                        Err(e) => return Err(format!("Failed to parse 'MIN' from 'int:MIN-MAX': {e}")),
                        Ok(v) => v,
                    };

                    let max = match max_str.parse::<i64>() {
                        Err(e) => return Err(format!("Failed to parse 'MAX' from 'int:MIN-MAX': {e}")),
                        Ok(v) => v,
                    };

                    Ok(ValueGenerator::RandInt(min, max))
                }
            },
            "float" => match gen_field_content.split_once('-') {
                None => Err("Inclusive range should have 'min-max' format".to_string()),
                Some((min_str, max_str)) => {
                    let min = match min_str.parse::<f64>() {
                        Err(e) => return Err(format!("Failed to parse 'MIN' from 'float:MIN-MAX': {e}")),
                        Ok(v) => v,
                    };

                    let max = match max_str.parse::<f64>() {
                        Err(e) => return Err(format!("Failed to parse 'MAX' from 'float:MIN-MAX': {e}")),
                        Ok(v) => v,
                    };

                    Ok(ValueGenerator::RandFloat(min, max))
                }
            },
            _ => Err(format!("Unsupported value '{gen_field_type}:...'")),
        }
    }
}

/// The data of a Kafka Record, as generated by [`RecordGenerator`].
///
/// The `key` and `payload` value are `Vec<u8>`,
/// as this is the most basic form of data we can give to the Kafka Producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRecord {
    topic: String,
    key: Option<Vec<u8>>,
    payload: Option<Vec<u8>>,
    partition: Option<i32>,
    headers: HashMap<String, String>,
}

/// Utility to generate records.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordGenerator {
    /// Topic the record is destined to.
    destination_topic: String,

    /// Generators of the content that will go in the record key.
    key_generator: Option<ValueGenerator>,

    /// Offers memoization for the record `key_generator`, depending on the [`ValueGenerator`] used.
    /// This is used when the value that goes in the key doesn't change at every record generation.
    key_generated_content: Option<Vec<u8>>,

    /// Generators of the content that will go in the record payload.
    payload_generator: Option<ValueGenerator>,

    /// Offers memoization for the record `payload_generator`, depending on the [`ValueGenerator`] used.
    /// This is used when the value that goes in the payload doesn't change at every record generation.
    payload_generated_content: Option<Vec<u8>>,

    /// Headers that will be added to the record.
    headers: HashMap<String, String>,

    /// Topic partition the record is destined to.
    /// If absent, this will be left to the Kafka Producer partitioner to determine.
    destination_partition: Option<i32>,
}

impl RecordGenerator {
    pub fn new(destination_topic: String) -> RecordGenerator {
        RecordGenerator {
            destination_topic,
            key_generator: None,
            key_generated_content: None,
            payload_generator: None,
            payload_generated_content: None,
            headers: HashMap::new(),
            destination_partition: None,
        }
    }

    pub fn add_record_header(&mut self, k: String, v: String) {
        self.headers.insert(k, v);
    }

    pub fn set_key_field(&mut self, key_generator: ValueGenerator) -> Result<(), Error> {
        // Memoize content, if appropriate
        self.key_generated_content = match key_generator {
            ValueGenerator::Fixed(_) | ValueGenerator::File(_) => Some(key_generator.generate()?),
            _ => None,
        };

        self.key_generator = Some(key_generator);

        Ok(())
    }

    pub fn set_payload_field(&mut self, payload_generator: ValueGenerator) -> Result<(), Error> {
        // Memoize content, if appropriate
        self.payload_generated_content = match payload_generator {
            ValueGenerator::Fixed(_) | ValueGenerator::File(_) => Some(payload_generator.generate()?),
            _ => None,
        };

        self.payload_generator = Some(payload_generator);

        Ok(())
    }

    pub fn set_destination_partition(&mut self, partition: i32) {
        self.destination_partition = Some(partition);
    }

    pub fn generate_record(&self) -> Result<GeneratedRecord, Error> {
        let rec = GeneratedRecord {
            topic: self.destination_topic.clone(),
            key: if let Some(k_mem) = &self.key_generated_content {
                Some(k_mem.to_vec())
            } else if let Some(k) = &self.key_generator {
                Some(k.generate()?)
            } else {
                None
            },
            payload: if let Some(p_mem) = &self.payload_generated_content {
                Some(p_mem.to_vec())
            } else if let Some(p) = &self.payload_generator {
                Some(p.generate()?)
            } else {
                None
            },
            partition: self.destination_partition,
            headers: self.headers.clone(),
        };

        Ok(rec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_only() {
        let mut generator = RecordGenerator::new("a_topic_name".to_string());
        assert!(generator.set_payload_field(ValueGenerator::Fixed("a payload content".to_string())).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!("a_topic_name", rec.topic);
        assert_eq!(None, rec.key);
        assert_eq!("a payload content".as_bytes(), rec.payload.unwrap());
        assert!(rec.headers.is_empty());
        assert_eq!(None, rec.partition);

        generator.set_destination_partition(10);
        let rec = generator.generate_record().unwrap();
        assert_eq!(Some(10), rec.partition);
    }

    #[test]
    fn test_key_and_headers() {
        let mut generator = RecordGenerator::new("another_topic".to_string());
        assert!(generator.set_payload_field(ValueGenerator::Fixed("another payload".to_string())).is_ok());

        generator.add_record_header("k1".to_string(), "v1".to_string());
        generator.add_record_header("k2".to_string(), "v2".to_string());
        generator.add_record_header("k3".to_string(), "v3".to_string());

        assert!(generator.set_key_field(ValueGenerator::RandInt(10, 10)).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!("another_topic", rec.topic);
        assert_eq!(10u64.to_be_bytes().to_vec(), rec.key.unwrap());
        assert_eq!("another payload".as_bytes(), rec.payload.unwrap());

        assert_eq!(3, rec.headers.len());
        assert!(rec.headers.contains_key("k1"));
        assert!(rec.headers.contains_key("k2"));
        assert!(rec.headers.contains_key("k3"));

        assert_eq!(None, rec.partition);
    }

    #[test]
    fn test_file_payload() {
        let cargo_toml_path = PathBuf::from("./Cargo.toml");
        let mut generator = RecordGenerator::new("topic_zzz".to_string());
        assert!(generator.set_payload_field(ValueGenerator::File(cargo_toml_path.clone())).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!("topic_zzz", rec.topic);
        assert_eq!(None, rec.key);
        let mut f = File::open(cargo_toml_path).unwrap();
        let mut cargo_toml_content = Vec::new();
        f.read_to_end(&mut cargo_toml_content).unwrap();

        assert_eq!(cargo_toml_content, rec.payload.unwrap());
        assert!(rec.headers.is_empty());
        assert_eq!(None, rec.partition);
    }

    #[test]
    fn test_randomizers() {
        let mut generator = RecordGenerator::new("topic".to_string());
        assert!(generator.set_key_field(ValueGenerator::RandBytes(20)).is_ok());
        assert!(generator.set_payload_field(ValueGenerator::RandAlphaNum(20)).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!(20, rec.key.unwrap().len());
        assert!(from_utf8(rec.payload.unwrap().as_slice()).is_ok());

        assert!(generator.set_payload_field(ValueGenerator::RandInt(123, 125)).is_ok());
        assert!(generator.set_key_field(ValueGenerator::RandFloat(1.5, 2.0)).is_ok());

        let rec = generator.generate_record().unwrap();
        let rec_key = f64::from_be_bytes(rec.key.unwrap().as_slice().try_into().unwrap());
        assert!((1.5..=2.0).contains(&rec_key));

        let rec_payload = i64::from_be_bytes(rec.payload.unwrap().as_slice().try_into().unwrap());
        assert!((123..=125).contains(&rec_payload));
    }
}
