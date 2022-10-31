# Kafkesc - Ksunami :ocean:

_**Waves of Kafka Records!**_

[![Build + Test](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml/badge.svg)](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml)

## :grey_question: Why?

Ksunami is a command line tool to produce volumes of (dummy) records against a [Kafka](https://kafka.apache.org/) cluster.

If you are experimenting with scalability and latency against Kafka, and are looking for ways to reproduce a continues
flow of records, following a very specific traffic pattern that repeats periodically, **Ksunami** is the tool for you.

Ksunami offers a way to set up the production of records, expressing the scenario as a sequence of "phases"
that repeat indefinitely.

## :bulb: Features

* Production described in 4 "phases" that repeat in circle: `min`, `up`, `max` and `down`
* All phases are configurable in terms of _seconds_ (duration) and _records per second_ (workload) 
* `up` and `down` can be one of many transitions, each with a specific "shape" (ex. `linear`, `ease-in`, `spike-out`, ...)
* Records `key` and `payload` are configurable with fixed, from-file and randomly-generated values
* Records headers can be added to each record
* Kafka producer is fully configurable, including selecting a partitioner

## Get started

### Cargo install

TODO

### Manually

TODO

## Core concepts

### :traffic_light: The 4 phases

TODO

### :roller_coaster: Transitions

TODO

## Usage

To begin, start with `ksunami -h` or `ksunami --help` for the short and long versions of the usage instructions.
_Go ahead, I'll wait!_.

### :wrench: Configuring the Producer

Additional to the obvious `-b, --brokers` for the bootstrap brokers, and `--client-id` for the client identifier,
it's possible to fine tune the Provider via `--partitioner` and `-c, --config`.

Possible values for the `--partitioner` argument are:

* `random`: Random distribution
* `consistent`: CRC32 hash of key (Empty and NULL keys are mapped to single partition)
* `consistent_random` (default): CRC32 hash of key (Empty and NULL keys are randomly partitioned)
* `murmur2`: Java Producer compatible Murmur2 hash of key (NULL keys are mapped to single partition)
* `murmur2_random`: Java Producer compatible Murmur2 hash of key (NULL keys are randomly partitioned). 
  This is functionally equivalent to the default partitioner in the Java Producer
* `fnv1a`: FNV-1a hash of key (NULL keys are mapped to single partition)
* `fnv1a_random`: FNV-1a hash of key (NULL keys are randomly partitioned)

For example, to use a _purely random partitioner_:

```shell
$ ksunami ... --partitioner random ...
```

As per `-c,--config`, all the values supported by producers of the
[librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md)
library are supported, as Ksunami is based on it.

For example, to set a _200ms producer lingering_ and to _limit the number of producer send retries to 5_:

```shell
$ ksunami ... -c linger.ms:200 ... --config message.send.max.retries:5 ...
```

### :factory: What goes into each Record

You can configure the content of each record produced by Ksunami:

* `-t, --topic <TOPIC>`: the destination topic to send the record to
* `-k, --key <KEY_TYPE:INPUT>` (optional): the key of the record
* `-p, --payload <PAYLOAD_TYPE:INPUT>` (optional): the payload of the record
* `--partition <PARTITION>` (optional): the specific partition inside the destination topic
* `--head <HEAD_KEY:HEAD_VAL>` (optional): one (or more) header(s) to decorate the record with

While for `--topic`, `--partition` and `--head` the input is pretty self-explanatory, `--key` and `--payload` support
a richer set of options.

The supported `KEY_TYPE/PAYLOAD_TYPE` are:

* `string:STR`: `STR` is a plain string
* `file:PATH`: `PATH` is a path to an existing file
* `alpha:LENGTH`: `LENGTH` is the length of a random alphanumeric string
* `bytes:LENGTH`: `LENGTH` is the length of a random bytes array
* `int:MIN-MAX`: `MIN` and `MAX` are limits of an inclusive range from which an integer number is picked
* `float:MIN-MAX`: `MIN` and `MAX` are limits of an inclusive range from which a float number is picked

This allows to have a degree of flexibility to the content that is placed inside records.

For example, to produce records where the _key is an integer between 1 and 1000_
and the _payload is a random sequence of 100 bytes_:

```shell
$ ksunami ... --key int:1-1000 --payload bytes:100 
```

### :chart_with_upwards_trend: How many Records

TODO

### :microphone: Log verbosity

Ksunami follows the long tradition of `-v/-q` to control the verbosity of it's logging:

* `-qq...  = OFF`
* `-q...   = ERROR`
* `<none>  = WARN`
* `-v      = INFO`
* `-vv     = DEBUG`
* `-vvv... = TRACE`

It uses [log](https://crates.io/crates/log) and [env_logger](https://crates.io/crates/env_logger),
and so logging can be configured and fine-tuned using the Environment Variable `KSUNAMI_LOG`.
Please take a look at [env_logger doc](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) for
more details.

## Examples

TODO

## TODOs

* [ ] Support for jitter in the size of keys/payload
* [ ] Support for jitter in the number of records per phase
* [ ] Support for jitter in the duration of each phase
* [ ] Surface "producer ack" config (?)
* [ ] Surface "compression" config (?)
* [ ] Support for sequential values for keys/payload (seq of ints? seq of strings? closed sequence? random amongst a set?)
* [ ] Publish a binary for Linux/macOS/Windows x AMD64/ARM64
* [ ] Publish a build via Homebrew

## License

[Apache License 2.0](./LICENSE)
