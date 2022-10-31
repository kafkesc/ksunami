# Kafkesc - Ksunami :ocean:

_**Waves of Kafka Records!**_

[![Build + Test](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml/badge.svg)](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml)

## Why?

Ksunami is a command line tool to produce volumes of (dummy) records against a [Kafka](https://kafka.apache.org/) cluster.

If you are experimenting with scalability and latency against your Kafka consumer applications, and are looking
for ways to reproduce uneven, _exceptional_ traffic patterns (i.e. _"spikes"_).

For example, imagine your cluster normally receives a steady, stable amount of traffic
(ex. 1,000 records/sec, equivalent to ~1MB/sec); but once a week you receive a spike that peaks at
100,000 records/sec (~102MB/sec). **How can you reproduce such a scenario on demand, so you can improve your system?**

Ksunami offers a way to setup the production of records, expressing the scenario as a sequence of "phases"
that repeat continuously until the process is interrupted. 

## Features

* Production described in 4 "phases": `min`, `up`, `max` and `down`
* All phases are configurable in terms of _seconds_ (duration) and _records per second_ 
* `up` and `down` transitions are also configured as curves (ex. `linear`, `ease-in`, `spike-out`, ...)
* Records key and payload are configurable, with fixed, from-file and randomly-generated values
* Records headers can be added to each record
* Fully configurable Kafka producer, including the partitioner

## Install

## Core concepts

### :traffic_light: The 4 phases

### :roller_coaster: Transitions

### Key and Payload content

## Usage

To begin, start with `ksunami -h` or `ksunami --help` for the short and long versions of the usage instructions:
go ahead, I'll wait.

### :wrench: Configuring the Producer

Additional to the obvious `-b,--brokers` for the bootstrap brokers, and `--client-id` for the client identifier,
it's possible to fine tune the Provider via `--partitioner` and `-c,--config`.

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

As per `-c,--config`, all the values supported by producers of the [librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md)
library are supported, as Ksunami is based on it.

For example, to set a _200ms producer lingering_ and to _limit the number of producer send retries to 5_:

```shell
$ ksunami ... -c linger.ms:200 ... --config message.send.max.retries:5 ...
```

### :factory: What goes into each Record

### :chart_with_upwards_trend: How many Records

### :microphone: Log verbosity

## Examples

## TODOs

* [ ] Support for jitter in the size of keys/payload
* [ ] Support for jitter in the number of records per phase
* [ ] Support for jitter in the duration of each phase
* [ ] Surface "producer ack" config (?)
* [ ] Surface "compression" config (?)
* [ ] Support for sequential values for keys/payload (seq of ints? seq of strings? closed sequence? random amongst a set?)

## License

[Apache License 2.0](./LICENSE)
