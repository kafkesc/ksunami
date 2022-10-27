use flo_curves::bezier;
use flo_curves::*;

use crate::transition::Transition;

/// It represents the amount of "work" to do, at any given time.
/// Time itself is measured in seconds, from `0` to [`std::u64::MAX`].
///
/// The amount of work is expressed as a `u32`, referring to the amount of records/sec,
/// and it's retrieved via [`Workload::records_per_sec_at`].
///
/// The workload goes through 4 phases:
///
/// * [`WorkloadPhase::Min`], lasting `min_sec`
/// * [`WorkloadPhase::Up`], lasting `up_sec`
/// * [`WorkloadPhase::Max`], lasting `max_sec`
/// * [`WorkloadPhase::Down`], lasting `down_sec`
///
/// Given the input at construction time, the workload repeats over and over, with a period
/// equivalent to the sum of `min_sec + max_sec + up_sec + down_sec`: this means that
/// after the [`WorkloadPhase::Down`], the [`WorkloadPhase::Min`] starts again.
#[derive(Debug, Clone, PartialEq)]
pub struct Workload {
    /// Minimum amount of records per second
    min: u32,

    /// How long to stay at minimum records/sec before ramp-up
    min_sec: u32,

    /// Maximum amount of records per second
    max: u32,

    /// How long to stay at maximum records/sec, before ramp-down
    max_sec: u32,

    /// Ramp-up transition duration
    up_sec: u32,

    /// Bézier Curve describing the ramp-up transition.
    /// Present if [`up_transition`] is not [`Transition::None`].
    up_curve: Option<bezier::Curve<Coord2>>,

    /// Ramp-down transition duration
    down_sec: u32,

    /// Bézier Curve describing the ramp-down" transition.
    /// Present if [`down_transition`] is not [`Transition::None`].
    down_curve: Option<bezier::Curve<Coord2>>,
}

/// Describes the phases that a [`Workload`] goes through cyclically.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorkloadPhase {
    Min,
    Up,
    Max,
    Down,
}

impl Workload {
    pub fn new(
        min: u32,
        min_sec: u32,
        max: u32,
        max_sec: u32,
        up_transition: Transition,
        up_sec: u32,
        down_transition: Transition,
        down_sec: u32,
    ) -> Workload {
        // P0/P3 for the "up" phase
        let up_p0 = Coord2(min_sec as f64, min as f64);
        let up_p3 = Coord2((min_sec + up_sec) as f64, max as f64);

        // P0/P3 for the "down" phase
        let down_p0 = Coord2((min_sec + up_sec + max_sec) as f64, max as f64);
        let down_p3 = Coord2((min_sec + up_sec + max_sec + down_sec) as f64, min as f64);

        // "up" phase duration and curve, depend on the corresponding Transition
        let (up_sec, up_curve) = if up_transition != Transition::None {
            (up_sec, Some(bezier::Curve::from_points(up_p0, up_transition.ctrl_pts_up(up_p0, up_p3).unwrap(), up_p3)))
        } else {
            (0, None)
        };

        // "down" phase duration and curve, depend on the corresponding Transition
        let (down_sec, down_curve) = if down_transition != Transition::None {
            (
                down_sec,
                Some(bezier::Curve::from_points(
                    down_p0,
                    down_transition.ctrl_pts_down(down_p0, down_p3).unwrap(),
                    down_p3,
                )),
            )
        } else {
            (0, None)
        };

        Workload {
            min,
            min_sec,
            max,
            max_sec,
            up_sec,
            up_curve,
            down_sec,
            down_curve,
        }
    }

    /// How long the [`WorkloadPhase::Min`] lasts, in seconds
    pub fn min_duration_sec(&self) -> u32 {
        self.min_sec
    }

    /// How long before the [`WorkloadPhase::Max`] starts, in seconds
    pub fn before_max_duration_sec(&self) -> u32 {
        self.min_sec + self.up_sec
    }

    /// How long before the [`WorkloadPhase::Max`] ends, in seconds
    pub fn after_max_duration_sec(&self) -> u32 {
        self.min_sec + self.up_sec + self.max_sec
    }

    /// How long before the [`WorkloadPhase::Down`] ends, in seconds.
    ///
    /// This represents also the entire length of a "cycle" of workload:
    /// after [`WorkloadPhase::Down`] ends, the [`WorkloadPhase::Min`] starts agai.
    pub fn overall_duration_sec(&self) -> u32 {
        self.min_sec + self.up_sec + self.max_sec + self.down_sec
    }

    /// Normalizes the input `sec` from absolute to relative.
    ///
    /// The [`WorkloadPhase`]s repeat in a loop, so it takes any absolute seconds input, and
    /// convert it to it's relative position in the [`Workload`].
    fn normalize_sec(&self, sec: u64) -> u32 {
        // Time "loops", so we normalize the input to be never
        // greater than the total `duration_sec`.
        (sec % self.overall_duration_sec() as u64) as u32
    }

    /// Given the input `sec`, informs of what [`WorkloadPhase`] that is at.
    pub fn phase_at(&self, sec: u64) -> WorkloadPhase {
        let nor_sec = self.normalize_sec(sec);

        if nor_sec < self.min_duration_sec() {
            WorkloadPhase::Min
        } else if nor_sec < self.before_max_duration_sec() {
            WorkloadPhase::Up
        } else if nor_sec < self.after_max_duration_sec() {
            WorkloadPhase::Max
        } else {
            WorkloadPhase::Down
        }
    }

    /// Given the input `sec`, returns the number of records/sec that this `Workload` indicates.
    ///
    /// The [`WorkloadPhase`]s repeat in a loop, so it takes any absolute seconds input, and
    /// returns the amount of records/sec for that moment in time.
    pub fn records_per_sec_at(&self, sec: u64) -> u32 {
        let nor_sec = self.normalize_sec(sec);

        match self.phase_at(sec) {
            WorkloadPhase::Min => self.min,
            WorkloadPhase::Up => {
                // The corresponding Bézier `t` for `nor_sec` during "up" phase
                let nor_sec_t = (nor_sec - self.min_duration_sec()) as f64 / self.up_sec as f64;

                // Return the corresponding Y (amount of records per second) give `t` as X
                self.up_curve.unwrap().point_at_pos(nor_sec_t).y().round() as u32
            }
            WorkloadPhase::Max => self.max,
            WorkloadPhase::Down => {
                // The corresponding Bézier `t` for `nor_sec` during "down" phase
                let nor_sec_t = (nor_sec - self.after_max_duration_sec()) as f64 / self.down_sec as f64;

                // Return the corresponding Y (amount of records per second) give `t` as X
                self.down_curve.unwrap().point_at_pos(nor_sec_t).y().round() as u32
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::workload::WorkloadPhase::*;

    use super::*;

    #[test]
    fn test_up_linear_down_none() {
        let w = Workload::new(1, 20, 100, 5, Transition::Linear, 3, Transition::None, 0);

        // min_sec=20 + up_sec=3 + max_sec=5 + down_sec=0
        assert_eq!(28, w.overall_duration_sec());

        // min at 0-19
        for sec in 0u64..=19u64 {
            assert_eq!(1, w.records_per_sec_at(sec));
        }

        // up at 20-22
        assert_eq!(1, w.records_per_sec_at(20));
        assert_eq!(27, w.records_per_sec_at(21));
        assert_eq!(74, w.records_per_sec_at(22));

        // max at 23-27
        for sec in 23u64..=27u64 {
            assert_eq!(100, w.records_per_sec_at(sec));
        }

        // down_sec is 0, so it's time for min again
        for sec in 28u64..=47u64 {
            assert_eq!(1, w.records_per_sec_at(sec));
        }

        // up at 48-50
        assert_eq!(1, w.records_per_sec_at(48));
        assert_eq!(27, w.records_per_sec_at(49));
        assert_eq!(74, w.records_per_sec_at(50));

        // max at 51-55
        for sec in 51u64..=55u64 {
            assert_eq!(100, w.records_per_sec_at(sec));
        }
    }

    #[test]
    fn test_up_spike_out_down_ease_in() {
        let w = Workload::new(3, 60, 100, 5, Transition::SpikeOut, 20, Transition::EaseIn, 20);

        let mut occurrences = HashMap::new();

        let mut prev = 0;
        for sec in 0u64..(w.overall_duration_sec() as u64 * 10u64) {
            let curr = w.records_per_sec_at(sec);

            // Count the occurrences of a specific phase
            let sec_phase = w.phase_at(sec);
            occurrences.entry(sec_phase.clone()).and_modify(|counter| *counter += 1).or_insert(1);

            // Given the phase, check that the behaviour is what we expect
            match sec_phase {
                Min => {
                    assert_eq!(curr, 3);
                }
                Up => {
                    assert!(curr >= prev);
                }
                Max => {
                    assert_eq!(curr, 100);
                }
                Down => {
                    assert!(curr <= prev);
                }
            }

            prev = curr;
        }

        // Confirm the occurrences of each phase match expectations
        assert_eq!(600, occurrences.get(&Min).cloned().unwrap());
        assert_eq!(200, occurrences.get(&Up).cloned().unwrap());
        assert_eq!(50, occurrences.get(&Max).cloned().unwrap());
        assert_eq!(200, occurrences.get(&Down).cloned().unwrap());
    }
}
