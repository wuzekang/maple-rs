use std::f64;

const CUBIC_BEZIER_SPLINE_SAMPLES: usize = 11;
const MAX_NEWTON_ITERATIONS: usize = 4;
const BEZIER_EPSILON: f64 = 1e-7;

#[derive(Debug, Clone)]
pub struct CubicBezier {
    ax: f64,
    bx: f64,
    cx: f64,
    ay: f64,
    by: f64,
    cy: f64,
    start_gradient: f64,
    end_gradient: f64,
    range_min: f64,
    range_max: f64,
    spline_samples: [f64; CUBIC_BEZIER_SPLINE_SAMPLES],
    #[cfg(debug_assertions)]
    monotonically_increasing: bool,
}

fn to_finite(value: f64) -> f64 {
    if value.is_infinite() {
        if value > 0.0 {
            f64::MAX
        } else {
            f64::MIN
        }
    } else {
        value
    }
}

impl CubicBezier {
    pub fn new(p1x: f64, p1y: f64, p2x: f64, p2y: f64) -> Self {
        let (ax, bx, cx, ay, by, cy) = Self::init_coefficients(p1x, p1y, p2x, p2y);
        let (start_gradient, end_gradient) = Self::init_gradients(p1x, p1y, p2x, p2y);
        let (range_min, range_max) = Self::init_range(p1y, p2y, ay, by, cy);
        let spline_samples = Self::init_spline(ax, bx, cx);

        #[cfg(debug_assertions)]
        let monotonically_increasing = p1x >= 0.0 && p1x <= 1.0 && p2x >= 0.0 && p2x <= 1.0;

        CubicBezier {
            ax,
            bx,
            cx,
            ay,
            by,
            cy,
            start_gradient,
            end_gradient,
            range_min,
            range_max,
            spline_samples,
            #[cfg(debug_assertions)]
            monotonically_increasing,
        }
    }

    fn init_coefficients(p1x: f64, p1y: f64, p2x: f64, p2y: f64) -> (f64, f64, f64, f64, f64, f64) {
        let cx = 3.0 * p1x;
        let bx = 3.0 * (p2x - p1x) - cx;
        let ax = 1.0 - cx - bx;

        let cy = to_finite(3.0 * p1y);
        let by = to_finite(3.0 * (p2y - p1y) - cy);
        let ay = to_finite(1.0 - cy - by);

        (ax, bx, cx, ay, by, cy)
    }

    fn init_gradients(p1x: f64, p1y: f64, p2x: f64, p2y: f64) -> (f64, f64) {
        let start_gradient = if p1x > 0.0 {
            p1y / p1x
        } else if p1y == 0.0 && p2x > 0.0 {
            p2y / p2x
        } else if p1y == 0.0 && p2y == 0.0 {
            1.0
        } else {
            0.0
        };

        let end_gradient = if p2x < 1.0 {
            (p2y - 1.0) / (p2x - 1.0)
        } else if (p2y - 1.0).abs() < f64::EPSILON && p1x < 1.0 {
            (p1y - 1.0) / (p1x - 1.0)
        } else if (p2y - 1.0).abs() < f64::EPSILON && (p1y - 1.0).abs() < f64::EPSILON {
            1.0
        } else {
            0.0
        };

        (start_gradient, end_gradient)
    }

    fn init_range(p1y: f64, p2y: f64, ay: f64, by: f64, cy: f64) -> (f64, f64) {
        let mut range_min = 0.0;
        let mut range_max = 1.0;

        if (0.0..1.0).contains(&p1y) && (0.0..=1.0).contains(&p2y) {
            return (range_min, range_max);
        }

        let a = 3.0 * ay;
        let b = 2.0 * by;
        let c = cy;

        if a.abs() < BEZIER_EPSILON && b.abs() < BEZIER_EPSILON {
            return (range_min, range_max);
        }

        let (mut t1, mut t2) = (0.0, 0.0);
        if a.abs() < BEZIER_EPSILON {
            t1 = -c / b;
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return (range_min, range_max);
            }
            let sqrt_d = discriminant.sqrt();
            t1 = (-b + sqrt_d) / (2.0 * a);
            t2 = (-b - sqrt_d) / (2.0 * a);
        }

        let sample = |t| ((ay * t + by) * t + cy) * t;

        let mut sol1 = 0.0;
        let mut sol2 = 0.0;
        if t1 > 0.0 && t1 < 1.0 {
            sol1 = sample(t1);
        }
        if t2 > 0.0 && t2 < 1.0 {
            sol2 = sample(t2);
        }

        range_min = range_min.min(sol1).min(sol2);
        range_max = range_max.max(sol1).max(sol2);

        (range_min, range_max)
    }

    fn init_spline(ax: f64, bx: f64, cx: f64) -> [f64; CUBIC_BEZIER_SPLINE_SAMPLES] {
        let delta_t = 1.0 / (CUBIC_BEZIER_SPLINE_SAMPLES - 1) as f64;
        let mut samples = [0.0; CUBIC_BEZIER_SPLINE_SAMPLES];
        for (i, sample) in samples.iter_mut().enumerate() {
            let t = i as f64 * delta_t;
            *sample = ((ax * t + bx) * t + cx) * t;
        }
        samples
    }


    #[inline]
    fn sample_curve_y(&self, t: f64) -> f64 {
        to_finite(((self.ay * t + self.by) * t + self.cy) * t)
    }

    #[inline]
    fn sample_curve_x(&self, t: f64) -> f64 {
        ((self.ax * t + self.bx) * t + self.cx) * t
    }

    #[inline]
    fn sample_curve_derivative_x(&self, t: f64) -> f64 {
        (3.0 * self.ax * t + 2.0 * self.bx) * t + self.cx
    }

    #[inline]
    fn sample_curve_derivative_y(&self, t: f64) -> f64 {
        (3.0 * self.ay * t + 2.0 * self.by) * t + self.cy
    }

    fn solve_curve_x(&self, x: f64, epsilon: f64) -> f64 {
        #[cfg(debug_assertions)]
        debug_assert!(
            self.monotonically_increasing,
            "Bezier curve is not monotonically increasing"
        );

        let x = x.clamp(0.0, 1.0);
        let mut t0 = 0.0;
        let mut t1 = 0.0;
        let mut t2 = x;

        let delta_t = 1.0 / (CUBIC_BEZIER_SPLINE_SAMPLES - 1) as f64;
        for i in 1..CUBIC_BEZIER_SPLINE_SAMPLES {
            if x <= self.spline_samples[i] {
                t1 = i as f64 * delta_t;
                t0 = t1 - delta_t;
                let d = self.spline_samples[i] - self.spline_samples[i - 1];
                if d != 0.0 {
                    t2 = t0 + (x - self.spline_samples[i - 1]) / d * delta_t;
                } else {
                    t2 = t0;
                }
                break;
            }
        }

        let newton_epsilon = epsilon.min(BEZIER_EPSILON);
        for _ in 0..MAX_NEWTON_ITERATIONS {
            let x2 = self.sample_curve_x(t2) - x;
            if x2.abs() < newton_epsilon {
                return t2;
            }
            let d2 = self.sample_curve_derivative_x(t2);
            if d2.abs() < BEZIER_EPSILON {
                break;
            }
            t2 -= x2 / d2;
        }

        if (self.sample_curve_x(t2) - x).abs() < epsilon {
            return t2;
        }

        let mut t2 = (t0 + t1) / 2.0;
        let mut x2 = self.sample_curve_x(t2);
        let mut iter_count = 0;
        while x2.abs() - x > epsilon && iter_count < 100 {
            if x > x2 {
                t0 = t2;
            } else {
                t1 = t2;
            }
            t2 = (t0 + t1) / 2.0;
            x2 = self.sample_curve_x(t2);
            iter_count += 1;
        }

        t2
    }

    pub fn solve(&self, x: f64) -> f64 {
        self.solve_with_epsilon(x, BEZIER_EPSILON)
    }

    pub fn solve_with_epsilon(&self, x: f64, epsilon: f64) -> f64 {
        if x <= 0.0 {
            return to_finite(0.0 + self.start_gradient * x);
        }
        if x >= 1.0 {
            return to_finite(1.0 + self.end_gradient * (x - 1.0));
        }
        self.sample_curve_y(self.solve_curve_x(x, epsilon))
    }

    pub fn slope(&self, x: f64) -> f64 {
        self.slope_with_epsilon(x, BEZIER_EPSILON)
    }

    pub fn slope_with_epsilon(&self, x: f64, epsilon: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        let t = self.solve_curve_x(x, epsilon);
        let dx = self.sample_curve_derivative_x(t);
        let dy = self.sample_curve_derivative_y(t);
        if dx == 0.0 && dy == 0.0 {
            0.0
        } else {
            to_finite(dy / dx)
        }
    }

    pub fn x1(&self) -> f64 {
        self.cx / 3.0
    }

    pub fn y1(&self) -> f64 {
        self.cy / 3.0
    }

    pub fn x2(&self) -> f64 {
        (self.bx + self.cx) / 3.0 + self.x1()
    }

    pub fn y2(&self) -> f64 {
        (self.by + self.cy) / 3.0 + self.y1()
    }
}