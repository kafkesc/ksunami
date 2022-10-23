use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;
use std::str::from_utf8;

use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};

/// Describes a possible field type used in [`RecordGenerator`].
///
/// Specifically, this is used for the [`RecordGenerator::key_field`] and [`RecordGenerator::payload_field`],
/// to specify what content should be generated for those fields when a Kafka record is generated
/// (via [`RecordGenerator::generate_record`].
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum GeneratorField<'a> {
    /// A user provided string.
    Fixed(&'a str),

    /// The content of a file.
    File(&'a Path),

    /// A random alphanumeric string.
    RandAlpha(usize),

    /// A random bytes array.
    RandBytes(usize),

    /// A random (signed) integer.
    RandInt(i64, i64),

    /// A random float.
    RandFloat(f64, f64),
}

impl<'a> GeneratorField<'a> {
    fn generate_value(&self) -> Result<Vec<u8>, Error> {
        match self {
            GeneratorField::Fixed(s) => Ok(s.as_bytes().to_vec()),
            GeneratorField::File(bp) => {
                debug!("Loading content of file {:?}", bp);
                let mut f = File::open(bp)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;

                Ok(buf)
            }
            GeneratorField::RandAlpha(l) => {
                let rand_alpha = Alphanumeric.sample_string(&mut thread_rng(), *l);

                Ok(rand_alpha.as_bytes().to_vec())
            }
            GeneratorField::RandBytes(l) => {
                let random_bytes: Vec<u8> = (0..*l).map(|_| thread_rng().gen::<u8>()).collect();

                Ok(random_bytes)
            }
            GeneratorField::RandInt(min, max) => {
                let random_int = thread_rng().gen_range(*min..=*max);

                Ok(random_int.to_be_bytes().to_vec())
            }
            GeneratorField::RandFloat(min, max) => {
                let random_float = thread_rng().gen_range(*min..=*max);

                Ok(random_float.to_be_bytes().to_vec())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRecord {
    topic: String,
    key: Option<Vec<u8>>,
    payload: Vec<u8>,
    partition: Option<i32>,
    headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordGenerator<'a> {
    topic: String,

    key_field: Option<GeneratorField<'a>>,
    /// Only set if [`RecordGenerator::key_field`] is a [`GeneratorField::File`] (memoization).
    key_field_file_content: Option<Vec<u8>>,

    payload_field: GeneratorField<'a>,
    /// Only set if [`RecordGenerator::payload_field`] is a [`GeneratorField::File`] (memoization).
    payload_field_file_content: Option<Vec<u8>>,

    headers: HashMap<String, String>,

    destination_partition: Option<i32>,
}

impl<'a> RecordGenerator<'a> {
    pub fn new(topic: String, payload_field: GeneratorField<'a>) -> Result<RecordGenerator, Error> {
        let payload_field_file_content = if let GeneratorField::File(_) = payload_field {
            Some(payload_field.generate_value()?)
        } else {
            None
        };

        Ok(RecordGenerator {
            topic,
            key_field: None,
            key_field_file_content: None,
            payload_field,
            payload_field_file_content,
            headers: HashMap::new(),
            destination_partition: None,
        })
    }

    pub fn add_record_header(&mut self, k: String, v: String) {
        self.headers.insert(k, v);
    }

    pub fn set_key_field(&mut self, key_field: GeneratorField<'a>) -> Result<(), Error> {
        let key_field_file_content = if let GeneratorField::File(_) = key_field {
            Some(key_field.generate_value()?)
        } else {
            None
        };

        self.key_field = Some(key_field);
        self.key_field_file_content = key_field_file_content;

        Ok(())
    }

    pub fn set_payload_field(&mut self, payload_field: GeneratorField<'a>) -> Result<(), Error> {
        let payload_field_file_content = if let GeneratorField::File(_) = payload_field {
            Some(payload_field.generate_value()?)
        } else {
            None
        };

        self.payload_field = payload_field;
        self.payload_field_file_content = payload_field_file_content;

        Ok(())
    }

    pub fn set_destination_partition(&mut self, partition: i32) {
        self.destination_partition = Some(partition);
    }

    pub fn generate_record(&self) -> Result<GeneratedRecord, Error> {
        let rec = GeneratedRecord {
            topic: self.topic.clone(),
            key: if let Some(k) = &self.key_field {
                if let GeneratorField::File(_) = k {
                    self.key_field_file_content.clone()
                } else {
                    Some(k.generate_value()?)
                }
            } else {
                None
            },
            payload: if let GeneratorField::File(_) = self.payload_field {
                self.payload_field_file_content.clone().unwrap()
            } else {
                self.payload_field.generate_value()?
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
        let mut generator =
            RecordGenerator::new("a_topic_name".to_string(), GeneratorField::Fixed("a payload content")).unwrap();

        let rec = generator.generate_record().unwrap();
        assert_eq!("a_topic_name", rec.topic);
        assert_eq!(None, rec.key);
        assert_eq!("a payload content".as_bytes(), rec.payload);
        assert!(rec.headers.is_empty());
        assert_eq!(None, rec.partition);

        generator.set_destination_partition(10);
        let rec = generator.generate_record().unwrap();
        assert_eq!(Some(10), rec.partition);
    }

    #[test]
    fn test_key_and_headers() {
        let mut generator =
            RecordGenerator::new("another_topic".to_string(), GeneratorField::Fixed("another payload")).unwrap();

        generator.add_record_header("k1".to_string(), "v1".to_string());
        generator.add_record_header("k2".to_string(), "v2".to_string());
        generator.add_record_header("k3".to_string(), "v3".to_string());

        assert!(generator.set_key_field(GeneratorField::RandInt(10, 10)).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!("another_topic", rec.topic);
        assert_eq!(10u64.to_be_bytes().to_vec(), rec.key.unwrap());
        assert_eq!("another payload".as_bytes(), rec.payload);

        assert_eq!(3, rec.headers.len());
        assert!(rec.headers.contains_key("k1"));
        assert!(rec.headers.contains_key("k2"));
        assert!(rec.headers.contains_key("k3"));

        assert_eq!(None, rec.partition);
    }

    #[test]
    fn test_file_payload() {
        let cargo_toml_path = Path::new("./Cargo.toml");
        let generator = RecordGenerator::new("topic_zzz".to_string(), GeneratorField::File(cargo_toml_path)).unwrap();

        let rec = generator.generate_record().unwrap();
        assert_eq!("topic_zzz", rec.topic);
        assert_eq!(None, rec.key);
        let mut f = File::open(cargo_toml_path).unwrap();
        let mut cargo_toml_content = Vec::new();
        f.read_to_end(&mut cargo_toml_content).unwrap();

        assert_eq!(cargo_toml_content, rec.payload);
        assert!(rec.headers.is_empty());
        assert_eq!(None, rec.partition);
    }

    #[test]
    fn test_randomizers() {
        let mut generator = RecordGenerator::new("topic".to_string(), GeneratorField::RandAlpha(20)).unwrap();
        assert!(generator.set_key_field(GeneratorField::RandBytes(20)).is_ok());

        let rec = generator.generate_record().unwrap();
        assert_eq!(20, rec.key.unwrap().len());
        assert_eq!(20, rec.payload.len());
        assert!(from_utf8(rec.payload.as_slice()).is_ok());

        let mut generator = RecordGenerator::new("topic".to_string(), GeneratorField::RandInt(123, 125)).unwrap();
        assert!(generator.set_key_field(GeneratorField::RandFloat(1.5, 2.0)).is_ok());

        let rec = generator.generate_record().unwrap();
        let rec_key = f64::from_be_bytes(rec.key.unwrap().as_slice().try_into().unwrap());
        assert!((1.5..=2.0).contains(&rec_key));

        let rec_payload = i64::from_be_bytes(rec.payload.as_slice().try_into().unwrap());
        assert!((123..=125).contains(&rec_payload));
    }
}
