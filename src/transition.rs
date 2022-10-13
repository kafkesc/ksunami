use clap::ValueEnum;
use flo_curves::{Coord2, Coordinate2D};

/// It represents a passing from the "min" phase to the "max" phase (i.e. "up" phase), or vice-versa (i.e. "down" phase).
///
/// A `Transition` is a name we give to a pair of control points used by Bézier Curves.
/// We use [Cubic Bézier curves](https://en.wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves)
/// to describe the transition: given the control points P0, P1, P2 and P3 of a Cubic Bézier,
/// each Transition represents `P1` and `P2` - the 2 middle-control-points of the curve.
///
/// `P0` and `P3` are instead defined by the phase we are in:
/// "up" phase means that `P0` will assume the "min" values, while `P3` will assume the "max" values;
/// "down" phase it's the reciprocal.
///
/// Note that the values of `P1` and `P2` in the documentation below are expressed at `t` values of a
/// Bézier curve (i.e. `0 <= t <= 1`): they are mapped to the final control points,
/// based on the bounding box of `P0` and `P3`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Transition {
    /// Immediate transition, with no in-between values.
    None,

    /// Linear transition, constant increments between values.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (0,0) and P2 = (1,1)
    /// * down: P1 = (0,1) and P2 = (1,0)
    Linear,

    /// Slow increment at the beginning, accelerates half way through until the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (.5,0) and P2 = (1,1)
    /// * down: P1 = (.5,1) and P2 = (1,0)
    EaseIn,

    /// Fast increment at the beginning, decelerates half way through until the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (0,0) and P2 = (.5,1)
    /// * down: P1 = (0,1) and P2 = (.5,0)
    EaseOut,

    /// Slow increment at the beginning, accelerates half way, decelerates at the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (.5,0) and P2 = (.5,1)
    /// * down: P1 = (.5,1) and P2 = (.5,0)
    EaseInOut,

    /// Fastest increment at the beginning, slowest deceleration close to the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (0,1) and P2 = (0,1)
    /// * down: P1 = (1,1) and P2 = (1,1)
    SpikeIn,

    /// Slowest increment at the beginning, fastest acceleration close to the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (1,0) and P2 = (1,0)
    /// * down: P1 = (0,0) and P2 = (0,0)
    SpikeOut,

    /// Fastest increment at the beginning, slow half way, fastest acceleration close to the end.
    ///
    /// Cubic Bézier P1 and P2 control points:
    ///
    /// * up:   P1 = (0,1) and P2 = (1,0)
    /// * down: P1 = (0,0) and P2 = (1,1)
    SpikeInOut,
}

impl Transition {
    pub fn ctrl_pts_up(&self, p0: Coord2, p3: Coord2) -> Option<(Coord2, Coord2)> {
        let (p1_t, p2_t) = match *self {
            Transition::None => {
                return None;
            }

            Transition::Linear => (Coord2(0., 0.), Coord2(1., 1.)),

            Transition::EaseIn => (Coord2(0.5, 0.), Coord2(1., 1.)),
            Transition::EaseOut => (Coord2(0., 0.), Coord2(0.5, 1.)),
            Transition::EaseInOut => (Coord2(0.5, 0.), Coord2(0.5, 1.)),

            Transition::SpikeIn => (Coord2(0., 1.), Coord2(0., 1.)),
            Transition::SpikeOut => (Coord2(1., 0.), Coord2(1., 0.)),
            Transition::SpikeInOut => (Coord2(0., 1.), Coord2(1., 0.)),
        };

        Some(map_p1t_p2t_to_p0_p3(p0, p3, p1_t, p2_t))
    }

    pub fn ctrl_pts_down(&self, p0: Coord2, p3: Coord2) -> Option<(Coord2, Coord2)> {
        let (p1_t, p2_t) = match *self {
            Transition::None => {
                return None;
            }

            Transition::Linear => (Coord2(0., 1.), Coord2(1., 0.)),

            Transition::EaseIn => (Coord2(0.5, 1.), Coord2(1., 0.)),
            Transition::EaseOut => (Coord2(0., 1.), Coord2(0.5, 0.)),
            Transition::EaseInOut => (Coord2(0.5, 1.), Coord2(0.5, 0.)),

            Transition::SpikeIn => (Coord2(1., 1.), Coord2(1., 1.)),
            Transition::SpikeOut => (Coord2(0., 0.), Coord2(0., 0.)),
            Transition::SpikeInOut => (Coord2(0., 0.), Coord2(1., 1.)),
        };

        Some(map_p1t_p2t_to_p0_p3(p0, p3, p1_t, p2_t))
    }
}

/// Find the control points `P1` and `P2`, between `P0` and `P3`, using the `t` value of `P1` and `P2`.
fn map_p1t_p2t_to_p0_p3(p0: Coord2, p3: Coord2, p1_t: Coord2, p2_t: Coord2) -> (Coord2, Coord2) {
    (
        Coord2(
            between(p0.x(), p3.x(), p1_t.x()),
            between(p0.y(), p3.y(), p1_t.y()),
        ),
        Coord2(
            between(p0.x(), p3.x(), p2_t.x()),
            between(p0.y(), p3.y(), p2_t.y()),
        ),
    )
}

/// Find in-between value between `a` and `b`, by `t`.
///
/// Note that `t == 0` maps to the `min(a, b)`, while `t == 1` maps to the `max(a,b)`.
fn between(a: f64, b: f64, t: f64) -> f64 {
    let min_val = a.min(b);
    let max_val = a.max(b);

    min_val + ((max_val - min_val) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none() {
        assert_eq!(
            Transition::None.ctrl_pts_up(Coord2(0., 0.), Coord2(1., 1.)),
            None
        );
    }

    #[test]
    fn test_linear() {
        let (p1, p2) = Transition::Linear
            .ctrl_pts_up(Coord2(1., 2.), Coord2(30., 40.))
            .unwrap();
        assert_eq!(p1, Coord2(1., 2.));
        assert_eq!(p2, Coord2(30., 40.));

        let (p1, p2) = Transition::Linear
            .ctrl_pts_down(Coord2(1., 40.), Coord2(30., 2.))
            .unwrap();
        assert_eq!(p1, Coord2(1., 40.));
        assert_eq!(p2, Coord2(30., 2.));
    }

    #[test]
    fn test_ease_in() {
        let (p1, p2) = Transition::EaseIn
            .ctrl_pts_up(Coord2(1., 2.), Coord2(32., 44.))
            .unwrap();
        assert_eq!(p1, Coord2(16.5, 2.));
        assert_eq!(p2, Coord2(32., 44.));

        let (p1, p2) = Transition::EaseIn
            .ctrl_pts_down(Coord2(1., 44.), Coord2(32., 2.))
            .unwrap();
        assert_eq!(p1, Coord2(16.5, 44.));
        assert_eq!(p2, Coord2(32., 2.));
    }

    #[test]
    fn test_ease_out() {
        let (p1, p2) = Transition::EaseOut
            .ctrl_pts_up(Coord2(1., 2.), Coord2(32., 44.))
            .unwrap();
        assert_eq!(p1, Coord2(1., 2.));
        assert_eq!(p2, Coord2(16.5, 44.));

        let (p1, p2) = Transition::EaseOut
            .ctrl_pts_down(Coord2(1., 44.), Coord2(32., 2.))
            .unwrap();
        assert_eq!(p1, Coord2(1., 44.));
        assert_eq!(p2, Coord2(16.5, 2.));
    }

    #[test]
    fn test_ease_in_out() {
        let (p1, p2) = Transition::EaseInOut
            .ctrl_pts_up(Coord2(11., 3.), Coord2(35., 50.))
            .unwrap();
        assert_eq!(p1, Coord2(23., 3.));
        assert_eq!(p2, Coord2(23., 50.));

        let (p1, p2) = Transition::EaseInOut
            .ctrl_pts_down(Coord2(11., 50.), Coord2(35., 3.))
            .unwrap();
        assert_eq!(p1, Coord2(23., 50.));
        assert_eq!(p2, Coord2(23., 3.));
    }

    #[test]
    fn test_spike_in() {
        let (p1, p2) = Transition::SpikeIn
            .ctrl_pts_up(Coord2(11., 3.), Coord2(35., 50.))
            .unwrap();
        assert_eq!(p1, Coord2(11., 50.));
        assert_eq!(p2, Coord2(11., 50.));

        let (p1, p2) = Transition::SpikeIn
            .ctrl_pts_down(Coord2(11., 50.), Coord2(35., 3.))
            .unwrap();
        assert_eq!(p1, Coord2(35., 50.));
        assert_eq!(p2, Coord2(35., 50.));
    }

    #[test]
    fn test_spike_out() {
        let (p1, p2) = Transition::SpikeOut
            .ctrl_pts_up(Coord2(11., 3.), Coord2(35., 50.))
            .unwrap();
        assert_eq!(p1, Coord2(35., 3.));
        assert_eq!(p2, Coord2(35., 3.));

        let (p1, p2) = Transition::SpikeOut
            .ctrl_pts_down(Coord2(11., 50.), Coord2(35., 3.))
            .unwrap();
        assert_eq!(p1, Coord2(11., 3.));
        assert_eq!(p2, Coord2(11., 3.));
    }

    #[test]
    fn test_spike_in_out() {
        let (p1, p2) = Transition::SpikeInOut
            .ctrl_pts_up(Coord2(11., 3.), Coord2(35., 50.))
            .unwrap();
        assert_eq!(p1, Coord2(11., 50.));
        assert_eq!(p2, Coord2(35., 3.));

        let (p1, p2) = Transition::SpikeInOut
            .ctrl_pts_down(Coord2(11., 50.), Coord2(35., 3.))
            .unwrap();
        assert_eq!(p1, Coord2(11., 3.));
        assert_eq!(p2, Coord2(35., 50.));
    }
}
