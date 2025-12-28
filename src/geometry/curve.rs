//! 2D Curve geometry for CAD operations
//!
//! Provides Bezier curves (quadratic and cubic), B-splines, and NURBS with
//! evaluation, subdivision, fitting, and arc length calculation.

use crate::core::*;
use crate::core::precision::lerp;
use crate::geometry::point::Point2D;
use nalgebra::Point2 as NPoint2;
use serde::{Deserialize, Serialize};

/// Bezier curve (quadratic or cubic)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierCurve {
    /// Control points (2 for linear, 3 for quadratic, 4 for cubic, etc.)
    pub control_points: Vec<Point2D>,
}

impl BezierCurve {
    /// Create a new Bezier curve from control points
    pub fn new(control_points: Vec<Point2D>) -> Self {
        assert!(control_points.len() >= 2, "Need at least 2 control points");
        Self { control_points }
    }

    /// Create a quadratic Bezier curve
    pub fn quadratic(p0: Point2D, p1: Point2D, p2: Point2D) -> Self {
        Self::new(vec![p0, p1, p2])
    }

    /// Create a cubic Bezier curve
    pub fn cubic(p0: Point2D, p1: Point2D, p2: Point2D, p3: Point2D) -> Self {
        Self::new(vec![p0, p1, p2, p3])
    }

    /// Get the degree of the curve
    pub fn degree(&self) -> usize {
        self.control_points.len() - 1
    }

    /// Evaluate the curve at parameter t [0, 1] using De Casteljau's algorithm
    pub fn evaluate(&self, t: f64) -> Point2D {
        de_casteljau(&self.control_points, t)
    }

    /// Get the derivative curve
    pub fn derivative(&self) -> Option<BezierCurve> {
        if self.control_points.len() < 2 {
            return None;
        }

        let n = self.degree() as f64;
        let derivative_points: Vec<Point2D> = self
            .control_points
            .windows(2)
            .map(|w| {
                let dx = (w[1].x - w[0].x) * n;
                let dy = (w[1].y - w[0].y) * n;
                Point2D::new(dx, dy)
            })
            .collect();

        Some(BezierCurve::new(derivative_points))
    }

    /// Get the tangent vector at parameter t
    pub fn tangent(&self, t: f64) -> Vector2 {
        if let Some(deriv) = self.derivative() {
            let tangent_point = deriv.evaluate(t);
            Vector2::new(tangent_point.x, tangent_point.y).normalize()
        } else {
            Vector2::new(1.0, 0.0)
        }
    }

    /// Get the normal vector at parameter t
    pub fn normal(&self, t: f64) -> Vector2 {
        let tangent = self.tangent(t);
        Vector2::new(-tangent.y, tangent.x)
    }

    /// Subdivide the curve at parameter t into two curves
    pub fn subdivide(&self, t: f64) -> (BezierCurve, BezierCurve) {
        let (left, right) = de_casteljau_subdivide(&self.control_points, t);
        (BezierCurve::new(left), BezierCurve::new(right))
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<BoundingBox2> {
        let points: Vec<NPoint2<f64>> = self
            .control_points
            .iter()
            .map(|p| NPoint2::new(p.x, p.y))
            .collect();
        BoundingBox2::from_points(&points)
    }

    /// Approximate arc length using adaptive subdivision
    pub fn arc_length(&self, tolerance: f64) -> f64 {
        arc_length_adaptive(self, 0.0, 1.0, tolerance)
    }

    /// Convert to polyline approximation
    pub fn to_polyline(&self, num_segments: usize) -> Vec<Point2D> {
        (0..=num_segments)
            .map(|i| {
                let t = i as f64 / num_segments as f64;
                self.evaluate(t)
            })
            .collect()
    }

    /// Elevate the degree of the curve
    pub fn elevate_degree(&self) -> BezierCurve {
        let n = self.degree();
        let mut new_points = Vec::with_capacity(n + 2);

        new_points.push(self.control_points[0]);

        for i in 1..=n {
            let t = i as f64 / (n + 1) as f64;
            let p = self.control_points[i - 1].lerp(&self.control_points[i], t);
            new_points.push(p);
        }

        new_points.push(self.control_points[n]);

        BezierCurve::new(new_points)
    }

    /// Find the closest point on the curve to a given point (approximate)
    pub fn closest_point(&self, point: &Point2D, samples: usize) -> (Point2D, f64) {
        let mut min_dist = f64::MAX;
        let mut min_t = 0.0;
        let mut min_point = self.evaluate(0.0);

        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let p = self.evaluate(t);
            let dist = point.distance_to(&p);
            if dist < min_dist {
                min_dist = dist;
                min_t = t;
                min_point = p;
            }
        }

        (min_point, min_t)
    }
}

/// B-spline curve
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BSpline {
    /// Control points
    pub control_points: Vec<Point2D>,
    /// Knot vector
    pub knots: Vec<f64>,
    /// Degree of the curve
    pub degree: usize,
}

impl BSpline {
    /// Create a new B-spline curve
    pub fn new(control_points: Vec<Point2D>, knots: Vec<f64>, degree: usize) -> Option<Self> {
        // Validate: knots.len() == control_points.len() + degree + 1
        if knots.len() != control_points.len() + degree + 1 {
            return None;
        }

        // Validate: knots are non-decreasing
        for i in 1..knots.len() {
            if knots[i] < knots[i - 1] {
                return None;
            }
        }

        Some(Self {
            control_points,
            knots,
            degree,
        })
    }

    /// Create a uniform B-spline
    pub fn uniform(control_points: Vec<Point2D>, degree: usize) -> Option<Self> {
        let n = control_points.len();
        let m = n + degree + 1;
        let knots: Vec<f64> = (0..m).map(|i| i as f64).collect();
        Self::new(control_points, knots, degree)
    }

    /// Create a clamped B-spline (knots are repeated at the ends)
    pub fn clamped(control_points: Vec<Point2D>, degree: usize) -> Option<Self> {
        let n = control_points.len();
        let m = n + degree + 1;

        let mut knots = Vec::with_capacity(m);

        // Repeat first knot (degree + 1) times
        for _ in 0..=degree {
            knots.push(0.0);
        }

        // Middle knots
        for i in 1..(n - degree) {
            knots.push(i as f64);
        }

        // Repeat last knot (degree + 1) times
        let last_value = (n - degree) as f64;
        for _ in 0..=degree {
            knots.push(last_value);
        }

        Self::new(control_points, knots, degree)
    }

    /// Evaluate the B-spline at parameter t using Cox-de Boor algorithm
    pub fn evaluate(&self, t: f64) -> Point2D {
        let t = t.clamp(self.knots[0], self.knots[self.knots.len() - 1]);

        // Find knot span
        let span = self.find_knot_span(t);

        // Compute basis functions
        let basis = self.basis_functions(span, t);

        // Compute point
        let mut x = 0.0;
        let mut y = 0.0;

        for i in 0..=self.degree {
            let cp_idx = span - self.degree + i;
            if cp_idx < self.control_points.len() {
                x += basis[i] * self.control_points[cp_idx].x;
                y += basis[i] * self.control_points[cp_idx].y;
            }
        }

        Point2D::new(x, y)
    }

    /// Find the knot span containing parameter t
    fn find_knot_span(&self, t: f64) -> usize {
        let n = self.control_points.len() - 1;

        // Special case: t at the end of knot vector
        if t >= self.knots[n + 1] {
            return n;
        }

        // Binary search
        let mut low = self.degree;
        let mut high = n + 1;

        while low < high {
            let mid = (low + high) / 2;
            if t < self.knots[mid] {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        low - 1
    }

    /// Compute basis functions using Cox-de Boor recursion
    fn basis_functions(&self, span: usize, t: f64) -> Vec<f64> {
        let mut basis = vec![0.0; self.degree + 1];
        let mut left = vec![0.0; self.degree + 1];
        let mut right = vec![0.0; self.degree + 1];

        basis[0] = 1.0;

        for j in 1..=self.degree {
            left[j] = t - self.knots[span + 1 - j];
            right[j] = self.knots[span + j] - t;
            let mut saved = 0.0;

            for r in 0..j {
                let temp = basis[r] / (right[r + 1] + left[j - r]);
                basis[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }

            basis[j] = saved;
        }

        basis
    }

    /// Get the parameter range [t_min, t_max]
    pub fn parameter_range(&self) -> (f64, f64) {
        (self.knots[self.degree], self.knots[self.control_points.len()])
    }

    /// Convert to polyline approximation
    pub fn to_polyline(&self, num_segments: usize) -> Vec<Point2D> {
        let (t_min, t_max) = self.parameter_range();
        (0..=num_segments)
            .map(|i| {
                let t = lerp(t_min, t_max, i as f64 / num_segments as f64);
                self.evaluate(t)
            })
            .collect()
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<BoundingBox2> {
        let points: Vec<NPoint2<f64>> = self
            .control_points
            .iter()
            .map(|p| NPoint2::new(p.x, p.y))
            .collect();
        BoundingBox2::from_points(&points)
    }
}

/// NURBS curve (Non-Uniform Rational B-Spline)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NurbsCurve {
    /// Control points
    pub control_points: Vec<Point2D>,
    /// Weights for each control point
    pub weights: Vec<f64>,
    /// Knot vector
    pub knots: Vec<f64>,
    /// Degree of the curve
    pub degree: usize,
}

impl NurbsCurve {
    /// Create a new NURBS curve
    pub fn new(
        control_points: Vec<Point2D>,
        weights: Vec<f64>,
        knots: Vec<f64>,
        degree: usize,
    ) -> Option<Self> {
        if control_points.len() != weights.len() {
            return None;
        }

        if knots.len() != control_points.len() + degree + 1 {
            return None;
        }

        // Validate: knots are non-decreasing
        for i in 1..knots.len() {
            if knots[i] < knots[i - 1] {
                return None;
            }
        }

        // Validate: all weights are positive
        if weights.iter().any(|&w| w <= 0.0) {
            return None;
        }

        Some(Self {
            control_points,
            weights,
            knots,
            degree,
        })
    }

    /// Create a clamped NURBS curve
    pub fn clamped(control_points: Vec<Point2D>, weights: Vec<f64>, degree: usize) -> Option<Self> {
        let n = control_points.len();
        let m = n + degree + 1;

        let mut knots = Vec::with_capacity(m);

        // Repeat first knot (degree + 1) times
        for _ in 0..=degree {
            knots.push(0.0);
        }

        // Middle knots
        for i in 1..(n - degree) {
            knots.push(i as f64);
        }

        // Repeat last knot (degree + 1) times
        let last_value = (n - degree) as f64;
        for _ in 0..=degree {
            knots.push(last_value);
        }

        Self::new(control_points, weights, knots, degree)
    }

    /// Evaluate the NURBS curve at parameter t
    pub fn evaluate(&self, t: f64) -> Point2D {
        let t = t.clamp(self.knots[0], self.knots[self.knots.len() - 1]);

        // Find knot span
        let span = self.find_knot_span(t);

        // Compute basis functions
        let basis = self.basis_functions(span, t);

        // Compute point using rational basis
        let mut x = 0.0;
        let mut y = 0.0;
        let mut w = 0.0;

        for i in 0..=self.degree {
            let cp_idx = span - self.degree + i;
            if cp_idx < self.control_points.len() {
                let weighted_basis = basis[i] * self.weights[cp_idx];
                x += weighted_basis * self.control_points[cp_idx].x;
                y += weighted_basis * self.control_points[cp_idx].y;
                w += weighted_basis;
            }
        }

        if w.abs() > EPSILON {
            Point2D::new(x / w, y / w)
        } else {
            Point2D::new(0.0, 0.0)
        }
    }

    /// Find the knot span containing parameter t
    fn find_knot_span(&self, t: f64) -> usize {
        let n = self.control_points.len() - 1;

        if t >= self.knots[n + 1] {
            return n;
        }

        let mut low = self.degree;
        let mut high = n + 1;

        while low < high {
            let mid = (low + high) / 2;
            if t < self.knots[mid] {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        low - 1
    }

    /// Compute basis functions
    fn basis_functions(&self, span: usize, t: f64) -> Vec<f64> {
        let mut basis = vec![0.0; self.degree + 1];
        let mut left = vec![0.0; self.degree + 1];
        let mut right = vec![0.0; self.degree + 1];

        basis[0] = 1.0;

        for j in 1..=self.degree {
            left[j] = t - self.knots[span + 1 - j];
            right[j] = self.knots[span + j] - t;
            let mut saved = 0.0;

            for r in 0..j {
                let temp = basis[r] / (right[r + 1] + left[j - r]);
                basis[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }

            basis[j] = saved;
        }

        basis
    }

    /// Get the parameter range
    pub fn parameter_range(&self) -> (f64, f64) {
        (self.knots[self.degree], self.knots[self.control_points.len()])
    }

    /// Convert to polyline approximation
    pub fn to_polyline(&self, num_segments: usize) -> Vec<Point2D> {
        let (t_min, t_max) = self.parameter_range();
        (0..=num_segments)
            .map(|i| {
                let t = lerp(t_min, t_max, i as f64 / num_segments as f64);
                self.evaluate(t)
            })
            .collect()
    }

    /// Convert to B-spline if all weights are equal
    pub fn to_bspline(&self) -> Option<BSpline> {
        let first_weight = self.weights[0];
        if self.weights.iter().all(|&w| (w - first_weight).abs() < EPSILON) {
            BSpline::new(self.control_points.clone(), self.knots.clone(), self.degree)
        } else {
            None
        }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> Option<BoundingBox2> {
        let points: Vec<NPoint2<f64>> = self
            .control_points
            .iter()
            .map(|p| NPoint2::new(p.x, p.y))
            .collect();
        BoundingBox2::from_points(&points)
    }
}

// Helper functions

/// De Casteljau's algorithm for Bezier curve evaluation
fn de_casteljau(points: &[Point2D], t: f64) -> Point2D {
    let mut temp = points.to_vec();

    for _ in 0..points.len() - 1 {
        for j in 0..temp.len() - 1 {
            temp[j] = temp[j].lerp(&temp[j + 1], t);
        }
        temp.pop();
    }

    temp[0]
}

/// De Casteljau's algorithm with subdivision
fn de_casteljau_subdivide(points: &[Point2D], t: f64) -> (Vec<Point2D>, Vec<Point2D>) {
    let n = points.len();
    let mut pyramid = vec![points.to_vec()];

    for i in 1..n {
        let mut level = Vec::new();
        for j in 0..n - i {
            level.push(pyramid[i - 1][j].lerp(&pyramid[i - 1][j + 1], t));
        }
        pyramid.push(level);
    }

    let mut left = Vec::new();
    let mut right = Vec::new();

    for i in 0..n {
        left.push(pyramid[i][0]);
        right.push(pyramid[n - 1 - i][i]);
    }

    (left, right)
}

/// Adaptive arc length calculation
fn arc_length_adaptive(curve: &BezierCurve, t0: f64, t1: f64, tolerance: f64) -> f64 {
    let p0 = curve.evaluate(t0);
    let p1 = curve.evaluate(t1);
    let pm = curve.evaluate((t0 + t1) / 2.0);

    let linear_length = p0.distance_to(&p1);
    let curve_length = p0.distance_to(&pm) + pm.distance_to(&p1);

    if (curve_length - linear_length).abs() < tolerance {
        curve_length
    } else {
        let tm = (t0 + t1) / 2.0;
        arc_length_adaptive(curve, t0, tm, tolerance)
            + arc_length_adaptive(curve, tm, t1, tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_quadratic() {
        let curve = BezierCurve::quadratic(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 2.0),
            Point2D::new(2.0, 0.0),
        );
        assert_eq!(curve.degree(), 2);

        let p = curve.evaluate(0.5);
        assert!(p.approx_eq(&Point2D::new(1.0, 1.0)));
    }

    #[test]
    fn test_bezier_cubic() {
        let curve = BezierCurve::cubic(
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0, 1.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(1.0, 0.0),
        );
        assert_eq!(curve.degree(), 3);

        let start = curve.evaluate(0.0);
        let end = curve.evaluate(1.0);
        assert!(start.approx_eq(&Point2D::new(0.0, 0.0)));
        assert!(end.approx_eq(&Point2D::new(1.0, 0.0)));
    }

    #[test]
    fn test_bezier_subdivide() {
        let curve = BezierCurve::quadratic(
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 2.0),
            Point2D::new(2.0, 0.0),
        );

        let (left, right) = curve.subdivide(0.5);
        assert_eq!(left.control_points.len(), 3);
        assert_eq!(right.control_points.len(), 3);
    }

    #[test]
    fn test_bspline_clamped() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        let curve = BSpline::clamped(points.clone(), 2).unwrap();

        let (t_min, t_max) = curve.parameter_range();
        let start = curve.evaluate(t_min);
        let end = curve.evaluate(t_max);

        // Clamped B-spline should interpolate the first and last control points
        assert!(start.approx_eq(&points[0]));
        assert!(end.approx_eq(&points[points.len() - 1]));
    }

    #[test]
    fn test_nurbs_curve() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];
        let curve = NurbsCurve::clamped(points.clone(), weights, 2).unwrap();

        let (t_min, t_max) = curve.parameter_range();
        let start = curve.evaluate(t_min);
        let end = curve.evaluate(t_max);

        assert!(start.approx_eq(&points[0]));
        assert!(end.approx_eq(&points[points.len() - 1]));
    }

    #[test]
    fn test_nurbs_to_bspline() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0]; // Equal weights
        let nurbs = NurbsCurve::clamped(points, weights, 2).unwrap();

        assert!(nurbs.to_bspline().is_some());
    }
}
