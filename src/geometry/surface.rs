//! 3D Surface Geometry
//!
//! This module provides complete implementations of 3D parametric surfaces including
//! planes, Bezier surfaces, B-splines, and NURBS with trimming support.

use nalgebra::{Point2, Point3, Vector3, Matrix4, Unit};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Infinite plane in 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane3D {
    /// A point on the plane
    pub point: Point3<f64>,
    /// Normal vector to the plane
    pub normal: Unit<Vector3<f64>>,
}

impl Plane3D {
    /// Creates a new plane from a point and normal
    pub fn new(point: Point3<f64>, normal: Vector3<f64>) -> Self {
        assert!(normal.norm() > 0.0, "Normal must be non-zero");
        Self {
            point,
            normal: Unit::new_normalize(normal),
        }
    }

    /// Creates a plane from three points
    pub fn from_points(p1: Point3<f64>, p2: Point3<f64>, p3: Point3<f64>) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2);

        if normal.norm() < 1e-10 {
            return None; // Points are collinear
        }

        Some(Self::new(p1, normal))
    }

    /// Creates the XY plane at Z = 0
    pub fn xy() -> Self {
        Self::new(Point3::origin(), Vector3::z())
    }

    /// Creates the XZ plane at Y = 0
    pub fn xz() -> Self {
        Self::new(Point3::origin(), Vector3::y())
    }

    /// Creates the YZ plane at X = 0
    pub fn yz() -> Self {
        Self::new(Point3::origin(), Vector3::x())
    }

    /// Signed distance from a point to the plane
    pub fn distance_to_point(&self, point: &Point3<f64>) -> f64 {
        let v = point - self.point;
        v.dot(self.normal.as_ref())
    }

    /// Projects a point onto the plane
    pub fn project_point(&self, point: &Point3<f64>) -> Point3<f64> {
        let distance = self.distance_to_point(point);
        point - self.normal.as_ref() * distance
    }

    /// Checks if a point lies on the plane (within tolerance)
    pub fn contains_point(&self, point: &Point3<f64>, tolerance: f64) -> bool {
        self.distance_to_point(point).abs() < tolerance
    }

    /// Finds the intersection point with a line segment
    pub fn intersect_line(&self, start: &Point3<f64>, end: &Point3<f64>) -> Option<Point3<f64>> {
        let direction = end - start;
        let denom = direction.dot(self.normal.as_ref());

        if denom.abs() < 1e-10 {
            return None; // Line is parallel to plane
        }

        let t = (self.point - start).dot(self.normal.as_ref()) / denom;

        if t < 0.0 || t > 1.0 {
            return None; // Intersection outside line segment
        }

        Some(start + direction * t)
    }

    /// Transforms the plane by a matrix
    pub fn transform(&self, matrix: &Matrix4<f64>) -> Self {
        let new_point = matrix.transform_point(&self.point);
        let new_normal = matrix.transform_vector(self.normal.as_ref());

        Self {
            point: new_point,
            normal: Unit::new_normalize(new_normal),
        }
    }

    /// Returns the D coefficient of the plane equation Ax + By + Cz + D = 0
    pub fn d_coefficient(&self) -> f64 {
        -self.point.coords.dot(self.normal.as_ref())
    }
}

/// Parametric surface trait
pub trait ParametricSurface {
    /// Evaluates the surface at parameter (u, v)
    fn evaluate(&self, u: f64, v: f64) -> Point3<f64>;

    /// Computes the normal at parameter (u, v)
    fn normal(&self, u: f64, v: f64) -> Unit<Vector3<f64>>;

    /// Returns the valid parameter range
    fn parameter_range(&self) -> ((f64, f64), (f64, f64));

    /// Computes the partial derivative with respect to u
    fn partial_u(&self, u: f64, v: f64) -> Vector3<f64> {
        let epsilon = 1e-6;
        let p1 = self.evaluate(u - epsilon, v);
        let p2 = self.evaluate(u + epsilon, v);
        (p2 - p1) / (2.0 * epsilon)
    }

    /// Computes the partial derivative with respect to v
    fn partial_v(&self, u: f64, v: f64) -> Vector3<f64> {
        let epsilon = 1e-6;
        let p1 = self.evaluate(u, v - epsilon);
        let p2 = self.evaluate(u, v + epsilon);
        (p2 - p1) / (2.0 * epsilon)
    }
}

/// Bezier surface defined by a grid of control points
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierSurface {
    /// Control points in a 2D grid [u_index][v_index]
    pub control_points: Vec<Vec<Point3<f64>>>,
}

impl BezierSurface {
    /// Creates a new Bezier surface
    pub fn new(control_points: Vec<Vec<Point3<f64>>>) -> Self {
        assert!(!control_points.is_empty(), "Control points cannot be empty");
        assert!(
            !control_points[0].is_empty(),
            "Control points cannot be empty"
        );

        let v_count = control_points[0].len();
        for row in &control_points {
            assert_eq!(
                row.len(),
                v_count,
                "All rows must have the same number of control points"
            );
        }

        Self { control_points }
    }

    /// Returns the degree in the u direction
    pub fn degree_u(&self) -> usize {
        self.control_points.len() - 1
    }

    /// Returns the degree in the v direction
    pub fn degree_v(&self) -> usize {
        self.control_points[0].len() - 1
    }

    /// Bernstein polynomial
    fn bernstein(n: usize, i: usize, t: f64) -> f64 {
        let binom = Self::binomial(n, i) as f64;
        binom * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
    }

    /// Binomial coefficient
    fn binomial(n: usize, k: usize) -> usize {
        if k > n {
            return 0;
        }
        if k == 0 || k == n {
            return 1;
        }

        let mut result = 1;
        for i in 0..k.min(n - k) {
            result = result * (n - i) / (i + 1);
        }
        result
    }
}

impl ParametricSurface for BezierSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3<f64> {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let n = self.degree_u();
        let m = self.degree_v();

        let mut point = Point3::origin();

        for i in 0..=n {
            for j in 0..=m {
                let basis = Self::bernstein(n, i, u) * Self::bernstein(m, j, v);
                let cp = &self.control_points[i][j];
                point.x += basis * cp.x;
                point.y += basis * cp.y;
                point.z += basis * cp.z;
            }
        }

        point
    }

    fn normal(&self, u: f64, v: f64) -> Unit<Vector3<f64>> {
        let du = self.partial_u(u, v);
        let dv = self.partial_v(u, v);
        Unit::new_normalize(du.cross(&dv))
    }

    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        ((0.0, 1.0), (0.0, 1.0))
    }
}

/// B-spline surface with knot vectors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BSplineSurface {
    /// Control points grid
    pub control_points: Vec<Vec<Point3<f64>>>,
    /// Knot vector in u direction
    pub knots_u: Vec<f64>,
    /// Knot vector in v direction
    pub knots_v: Vec<f64>,
    /// Degree in u direction
    pub degree_u: usize,
    /// Degree in v direction
    pub degree_v: usize,
}

impl BSplineSurface {
    /// Creates a new B-spline surface
    pub fn new(
        control_points: Vec<Vec<Point3<f64>>>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
        degree_u: usize,
        degree_v: usize,
    ) -> Self {
        assert!(!control_points.is_empty());
        assert!(!control_points[0].is_empty());

        // Validate knot vectors
        assert_eq!(
            knots_u.len(),
            control_points.len() + degree_u + 1,
            "Invalid knot vector length in u direction"
        );
        assert_eq!(
            knots_v.len(),
            control_points[0].len() + degree_v + 1,
            "Invalid knot vector length in v direction"
        );

        Self {
            control_points,
            knots_u,
            knots_v,
            degree_u,
            degree_v,
        }
    }

    /// Creates a uniform B-spline surface
    pub fn uniform(
        control_points: Vec<Vec<Point3<f64>>>,
        degree_u: usize,
        degree_v: usize,
    ) -> Self {
        let nu = control_points.len();
        let nv = control_points[0].len();

        let knots_u = Self::uniform_knot_vector(nu, degree_u);
        let knots_v = Self::uniform_knot_vector(nv, degree_v);

        Self::new(control_points, knots_u, knots_v, degree_u, degree_v)
    }

    /// Generates a uniform knot vector
    fn uniform_knot_vector(n: usize, degree: usize) -> Vec<f64> {
        let m = n + degree + 1;
        let mut knots = vec![0.0; m];

        for i in 0..m {
            if i <= degree {
                knots[i] = 0.0;
            } else if i >= m - degree - 1 {
                knots[i] = 1.0;
            } else {
                knots[i] = (i - degree) as f64 / (m - 2 * degree - 1) as f64;
            }
        }

        knots
    }

    /// B-spline basis function (Cox-de Boor recursion)
    fn basis(&self, i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
        if p == 0 {
            return if t >= knots[i] && t < knots[i + 1] {
                1.0
            } else {
                0.0
            };
        }

        let mut left = 0.0;
        let denom_left = knots[i + p] - knots[i];
        if denom_left.abs() > 1e-10 {
            left = (t - knots[i]) / denom_left * self.basis(i, p - 1, t, knots);
        }

        let mut right = 0.0;
        let denom_right = knots[i + p + 1] - knots[i + 1];
        if denom_right.abs() > 1e-10 {
            right = (knots[i + p + 1] - t) / denom_right * self.basis(i + 1, p - 1, t, knots);
        }

        left + right
    }
}

impl ParametricSurface for BSplineSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3<f64> {
        let mut point = Point3::origin();

        for i in 0..self.control_points.len() {
            for j in 0..self.control_points[0].len() {
                let basis_u = self.basis(i, self.degree_u, u, &self.knots_u);
                let basis_v = self.basis(j, self.degree_v, v, &self.knots_v);
                let basis = basis_u * basis_v;

                let cp = &self.control_points[i][j];
                point.x += basis * cp.x;
                point.y += basis * cp.y;
                point.z += basis * cp.z;
            }
        }

        point
    }

    fn normal(&self, u: f64, v: f64) -> Unit<Vector3<f64>> {
        let du = self.partial_u(u, v);
        let dv = self.partial_v(u, v);
        Unit::new_normalize(du.cross(&dv))
    }

    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        let u_min = self.knots_u[self.degree_u];
        let u_max = self.knots_u[self.knots_u.len() - self.degree_u - 1];
        let v_min = self.knots_v[self.degree_v];
        let v_max = self.knots_v[self.knots_v.len() - self.degree_v - 1];

        ((u_min, u_max), (v_min, v_max))
    }
}

/// NURBS (Non-Uniform Rational B-Spline) surface with weights
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NurbsSurface {
    /// Control points grid
    pub control_points: Vec<Vec<Point3<f64>>>,
    /// Weights for each control point
    pub weights: Vec<Vec<f64>>,
    /// Knot vector in u direction
    pub knots_u: Vec<f64>,
    /// Knot vector in v direction
    pub knots_v: Vec<f64>,
    /// Degree in u direction
    pub degree_u: usize,
    /// Degree in v direction
    pub degree_v: usize,
    /// Trimming curves (optional)
    pub trimming_curves: Vec<TrimCurve>,
}

impl NurbsSurface {
    /// Creates a new NURBS surface
    pub fn new(
        control_points: Vec<Vec<Point3<f64>>>,
        weights: Vec<Vec<f64>>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
        degree_u: usize,
        degree_v: usize,
    ) -> Self {
        assert_eq!(control_points.len(), weights.len());
        assert_eq!(control_points[0].len(), weights[0].len());

        Self {
            control_points,
            weights,
            knots_u,
            knots_v,
            degree_u,
            degree_v,
            trimming_curves: Vec::new(),
        }
    }

    /// Adds a trimming curve
    pub fn add_trim_curve(&mut self, curve: TrimCurve) {
        self.trimming_curves.push(curve);
    }

    /// Checks if a parameter point is trimmed away
    pub fn is_trimmed(&self, u: f64, v: f64) -> bool {
        if self.trimming_curves.is_empty() {
            return false;
        }

        let point = Point2::new(u, v);

        // Use winding number algorithm
        let mut winding_number = 0;

        for curve in &self.trimming_curves {
            for i in 0..curve.points.len() {
                let p1 = &curve.points[i];
                let p2 = &curve.points[(i + 1) % curve.points.len()];

                if p1.y <= v {
                    if p2.y > v {
                        // Upward crossing
                        if Self::is_left(p1, p2, &point) > 0.0 {
                            winding_number += 1;
                        }
                    }
                } else {
                    if p2.y <= v {
                        // Downward crossing
                        if Self::is_left(p1, p2, &point) < 0.0 {
                            winding_number -= 1;
                        }
                    }
                }
            }
        }

        winding_number == 0 // Outside if winding number is 0
    }

    /// Helper for point-in-polygon test
    fn is_left(p0: &Point2<f64>, p1: &Point2<f64>, p2: &Point2<f64>) -> f64 {
        (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)
    }

    /// B-spline basis function
    fn basis(&self, i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
        if p == 0 {
            return if t >= knots[i] && t < knots[i + 1] {
                1.0
            } else {
                0.0
            };
        }

        let mut left = 0.0;
        let denom_left = knots[i + p] - knots[i];
        if denom_left.abs() > 1e-10 {
            left = (t - knots[i]) / denom_left * self.basis(i, p - 1, t, knots);
        }

        let mut right = 0.0;
        let denom_right = knots[i + p + 1] - knots[i + 1];
        if denom_right.abs() > 1e-10 {
            right = (knots[i + p + 1] - t) / denom_right * self.basis(i + 1, p - 1, t, knots);
        }

        left + right
    }
}

impl ParametricSurface for NurbsSurface {
    fn evaluate(&self, u: f64, v: f64) -> Point3<f64> {
        if self.is_trimmed(u, v) {
            // Return a marker point or handle trimmed regions differently
            return Point3::origin();
        }

        let mut numerator: Point3<f64> = Point3::origin();
        let mut denominator = 0.0;

        for i in 0..self.control_points.len() {
            for j in 0..self.control_points[0].len() {
                let basis_u = self.basis(i, self.degree_u, u, &self.knots_u);
                let basis_v = self.basis(j, self.degree_v, v, &self.knots_v);
                let weight = self.weights[i][j];
                let rational_basis = basis_u * basis_v * weight;

                let cp = &self.control_points[i][j];
                numerator.x += rational_basis * cp.x;
                numerator.y += rational_basis * cp.y;
                numerator.z += rational_basis * cp.z;
                denominator += rational_basis;
            }
        }

        if denominator.abs() < 1e-10 {
            return Point3::origin();
        }

        Point3::new(
            numerator.x / denominator,
            numerator.y / denominator,
            numerator.z / denominator,
        )
    }

    fn normal(&self, u: f64, v: f64) -> Unit<Vector3<f64>> {
        let du = self.partial_u(u, v);
        let dv = self.partial_v(u, v);
        let normal = du.cross(&dv);

        if normal.norm() < 1e-10 {
            return Unit::new_unchecked(Vector3::z());
        }

        Unit::new_normalize(normal)
    }

    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        let u_min = self.knots_u[self.degree_u];
        let u_max = self.knots_u[self.knots_u.len() - self.degree_u - 1];
        let v_min = self.knots_v[self.degree_v];
        let v_max = self.knots_v[self.knots_v.len() - self.degree_v - 1];

        ((u_min, u_max), (v_min, v_max))
    }
}

/// Trimming curve in parameter space
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimCurve {
    /// Points defining the trim boundary in (u, v) parameter space
    pub points: Vec<Point2<f64>>,
    /// Whether this is an outer or inner boundary
    pub is_outer: bool,
}

impl TrimCurve {
    /// Creates a new trim curve
    pub fn new(points: Vec<Point2<f64>>, is_outer: bool) -> Self {
        assert!(points.len() >= 3, "Trim curve must have at least 3 points");
        Self { points, is_outer }
    }

    /// Creates a rectangular trim curve
    pub fn rectangle(u_min: f64, u_max: f64, v_min: f64, v_max: f64) -> Self {
        Self::new(
            vec![
                Point2::new(u_min, v_min),
                Point2::new(u_max, v_min),
                Point2::new(u_max, v_max),
                Point2::new(u_min, v_max),
            ],
            true,
        )
    }

    /// Creates a circular trim curve
    pub fn circle(center_u: f64, center_v: f64, radius: f64, segments: usize) -> Self {
        let mut points = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = 2.0 * PI * (i as f64) / (segments as f64);
            points.push(Point2::new(
                center_u + radius * angle.cos(),
                center_v + radius * angle.sin(),
            ));
        }

        Self::new(points, false)
    }
}

/// Curvature analysis results
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceCurvature {
    /// Principal curvature in the first direction
    pub k1: f64,
    /// Principal curvature in the second direction
    pub k2: f64,
    /// Mean curvature
    pub mean: f64,
    /// Gaussian curvature
    pub gaussian: f64,
}

impl SurfaceCurvature {
    /// Computes curvature for a parametric surface at (u, v)
    pub fn compute<S: ParametricSurface>(surface: &S, u: f64, v: f64) -> Self {
        let eps = 1e-6;

        // First derivatives
        let su = surface.partial_u(u, v);
        let sv = surface.partial_v(u, v);

        // Second derivatives
        let suu = (surface.partial_u(u + eps, v) - surface.partial_u(u - eps, v)) / (2.0 * eps);
        let svv = (surface.partial_v(u, v + eps) - surface.partial_v(u, v - eps)) / (2.0 * eps);
        let suv = (surface.partial_u(u, v + eps) - surface.partial_u(u, v - eps)) / (2.0 * eps);

        let normal = surface.normal(u, v);
        let n = normal.as_ref();

        // First fundamental form coefficients
        let e = su.dot(&su);
        let f = su.dot(&sv);
        let g = sv.dot(&sv);

        // Second fundamental form coefficients
        let l = suu.dot(n);
        let m = suv.dot(n);
        let n_coef = svv.dot(n);

        // Principal curvatures
        let mean = (e * n_coef - 2.0 * f * m + g * l) / (2.0 * (e * g - f * f));
        let gaussian = (l * n_coef - m * m) / (e * g - f * f);

        let discriminant = (mean * mean - gaussian).max(0.0).sqrt();
        let k1 = mean + discriminant;
        let k2 = mean - discriminant;

        SurfaceCurvature {
            k1,
            k2,
            mean,
            gaussian,
        }
    }

    /// Returns true if the surface is flat at this point
    pub fn is_flat(&self, tolerance: f64) -> bool {
        self.k1.abs() < tolerance && self.k2.abs() < tolerance
    }

    /// Returns true if the surface is cylindrical at this point
    pub fn is_cylindrical(&self, tolerance: f64) -> bool {
        (self.k1.abs() < tolerance && self.k2.abs() > tolerance)
            || (self.k2.abs() < tolerance && self.k1.abs() > tolerance)
    }

    /// Returns true if the surface is spherical at this point
    pub fn is_spherical(&self, tolerance: f64) -> bool {
        (self.k1 - self.k2).abs() < tolerance && self.k1.abs() > tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_plane3d() {
        let plane = Plane3D::xy();

        assert_relative_eq!(
            plane.distance_to_point(&Point3::new(5.0, 3.0, 10.0)),
            10.0,
            epsilon = 1e-10
        );

        let projected = plane.project_point(&Point3::new(5.0, 3.0, 10.0));
        assert_relative_eq!(projected.z, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_bezier_surface() {
        let control_points = vec![
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ],
            vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 1.0, 1.0),
            ],
        ];

        let surface = BezierSurface::new(control_points);

        let p = surface.evaluate(0.0, 0.0);
        assert_relative_eq!(p.x, 0.0, epsilon = 1e-10);
        assert_relative_eq!(p.y, 0.0, epsilon = 1e-10);

        let p = surface.evaluate(1.0, 1.0);
        assert_relative_eq!(p.x, 1.0, epsilon = 1e-10);
        assert_relative_eq!(p.y, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_trim_curve() {
        let trim = TrimCurve::rectangle(0.2, 0.8, 0.2, 0.8);
        assert_eq!(trim.points.len(), 4);
    }

    #[test]
    fn test_nurbs_surface() {
        let control_points = vec![
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ],
            vec![
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 1.0, 0.0),
            ],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let knots_u = vec![0.0, 0.0, 1.0, 1.0];
        let knots_v = vec![0.0, 0.0, 1.0, 1.0];

        let surface = NurbsSurface::new(control_points, weights, knots_u, knots_v, 1, 1);

        let p = surface.evaluate(0.5, 0.5);
        assert_relative_eq!(p.x, 0.5, epsilon = 1e-10);
        assert_relative_eq!(p.y, 0.5, epsilon = 1e-10);
    }
}
