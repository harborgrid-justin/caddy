//! NURBS (Non-Uniform Rational B-Splines) curves and surfaces
//!
//! Provides enterprise-grade NURBS implementation with robust algorithms
//! for curve and surface evaluation, derivatives, and manipulation.

use crate::core::{Point3, Vector3, Point2, EPSILON};
use serde::{Deserialize, Serialize};


/// NURBS curve in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbsCurve {
    /// Degree of the curve
    pub degree: usize,
    /// Control points (4D homogeneous coordinates: x, y, z, w)
    pub control_points: Vec<[f64; 4]>,
    /// Knot vector
    pub knots: Vec<f64>,
}

impl NurbsCurve {
    /// Create a new NURBS curve
    pub fn new(degree: usize, control_points: Vec<Point3>, weights: Vec<f64>) -> Result<Self, NurbsError> {
        if control_points.len() != weights.len() {
            return Err(NurbsError::MismatchedControlPointsAndWeights);
        }

        if control_points.len() < degree + 1 {
            return Err(NurbsError::InsufficientControlPoints);
        }

        // Convert to homogeneous coordinates
        let homogeneous_points: Vec<[f64; 4]> = control_points
            .iter()
            .zip(weights.iter())
            .map(|(p, &w)| [p.x * w, p.y * w, p.z * w, w])
            .collect();

        // Generate uniform knot vector
        let knots = Self::generate_uniform_knots(degree, control_points.len());

        Ok(Self {
            degree,
            control_points: homogeneous_points,
            knots,
        })
    }

    /// Create a NURBS curve with explicit knot vector
    pub fn with_knots(
        degree: usize,
        control_points: Vec<Point3>,
        weights: Vec<f64>,
        knots: Vec<f64>,
    ) -> Result<Self, NurbsError> {
        if control_points.len() != weights.len() {
            return Err(NurbsError::MismatchedControlPointsAndWeights);
        }

        if knots.len() != control_points.len() + degree + 1 {
            return Err(NurbsError::InvalidKnotVector);
        }

        // Validate knot vector is non-decreasing
        for i in 1..knots.len() {
            if knots[i] < knots[i - 1] {
                return Err(NurbsError::InvalidKnotVector);
            }
        }

        // Convert to homogeneous coordinates
        let homogeneous_points: Vec<[f64; 4]> = control_points
            .iter()
            .zip(weights.iter())
            .map(|(p, &w)| [p.x * w, p.y * w, p.z * w, w])
            .collect();

        Ok(Self {
            degree,
            control_points: homogeneous_points,
            knots,
        })
    }

    /// Generate a uniform knot vector
    fn generate_uniform_knots(degree: usize, num_control_points: usize) -> Vec<f64> {
        let n = num_control_points + degree + 1;
        let mut knots = Vec::with_capacity(n);

        // Clamped uniform knot vector
        for i in 0..=degree {
            knots.push(0.0);
        }

        let interior_knots = num_control_points - degree;
        for i in 1..interior_knots {
            knots.push(i as f64 / interior_knots as f64);
        }

        for i in 0..=degree {
            knots.push(1.0);
        }

        knots
    }

    /// Evaluate the curve at parameter t
    pub fn evaluate(&self, t: f64) -> Result<Point3, NurbsError> {
        if t < self.knots[0] || t > *self.knots.last().unwrap() {
            return Err(NurbsError::ParameterOutOfRange);
        }

        let n = self.control_points.len();
        let p = self.degree;

        // Find knot span
        let span = self.find_span(t);

        // Compute basis functions
        let basis = self.basis_functions(span, t);

        // Compute curve point in homogeneous coordinates
        let mut point = [0.0, 0.0, 0.0, 0.0];
        for i in 0..=p {
            let idx = span - p + i;
            if idx < n {
                let b = basis[i];
                point[0] += b * self.control_points[idx][0];
                point[1] += b * self.control_points[idx][1];
                point[2] += b * self.control_points[idx][2];
                point[3] += b * self.control_points[idx][3];
            }
        }

        // Convert from homogeneous to Cartesian
        if point[3].abs() < EPSILON {
            return Err(NurbsError::DegeneratePoint);
        }

        Ok(Point3::new(
            point[0] / point[3],
            point[1] / point[3],
            point[2] / point[3],
        ))
    }

    /// Evaluate curve derivative at parameter t
    pub fn derivative(&self, t: f64, order: usize) -> Result<Vector3, NurbsError> {
        if order == 0 {
            return Ok(Vector3::zeros());
        }

        let dt = 1e-6;
        let p1 = self.evaluate(t)?;
        let p2 = self.evaluate(t + dt)?;

        Ok((p2 - p1) / dt)
    }

    /// Find the knot span containing parameter t
    fn find_span(&self, t: f64) -> usize {
        let n = self.control_points.len() - 1;
        let p = self.degree;

        if t >= self.knots[n + 1] {
            return n;
        }

        // Binary search
        let mut low = p;
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
        let p = self.degree;
        let mut basis = vec![0.0; p + 1];
        let mut left = vec![0.0; p + 1];
        let mut right = vec![0.0; p + 1];

        basis[0] = 1.0;

        for j in 1..=p {
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

    /// Tessellate the curve into line segments
    pub fn tessellate(&self, num_samples: usize) -> Result<Vec<Point3>, NurbsError> {
        let mut points = Vec::with_capacity(num_samples);
        let t_min = self.knots[0];
        let t_max = *self.knots.last().unwrap();

        for i in 0..num_samples {
            let t = t_min + (t_max - t_min) * (i as f64 / (num_samples - 1) as f64);
            points.push(self.evaluate(t)?);
        }

        Ok(points)
    }

    /// Get the curve's bounding box
    pub fn bounding_box(&self) -> (Point3, Point3) {
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for cp in &self.control_points {
            if cp[3].abs() > EPSILON {
                let x = cp[0] / cp[3];
                let y = cp[1] / cp[3];
                let z = cp[2] / cp[3];

                min.x = min.x.min(x);
                min.y = min.y.min(y);
                min.z = min.z.min(z);

                max.x = max.x.max(x);
                max.y = max.y.max(y);
                max.z = max.z.max(z);
            }
        }

        (min, max)
    }
}

/// NURBS surface in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurbsSurface {
    /// Degree in U direction
    pub degree_u: usize,
    /// Degree in V direction
    pub degree_v: usize,
    /// Control points grid (4D homogeneous coordinates)
    pub control_points: Vec<Vec<[f64; 4]>>,
    /// Knot vector in U direction
    pub knots_u: Vec<f64>,
    /// Knot vector in V direction
    pub knots_v: Vec<f64>,
}

impl NurbsSurface {
    /// Create a new NURBS surface
    pub fn new(
        degree_u: usize,
        degree_v: usize,
        control_points: Vec<Vec<Point3>>,
        weights: Vec<Vec<f64>>,
    ) -> Result<Self, NurbsError> {
        if control_points.len() != weights.len() {
            return Err(NurbsError::MismatchedControlPointsAndWeights);
        }

        let num_u = control_points.len();
        let num_v = control_points[0].len();

        // Validate dimensions
        for (cp_row, w_row) in control_points.iter().zip(weights.iter()) {
            if cp_row.len() != num_v || w_row.len() != num_v {
                return Err(NurbsError::MismatchedControlPointsAndWeights);
            }
        }

        if num_u < degree_u + 1 || num_v < degree_v + 1 {
            return Err(NurbsError::InsufficientControlPoints);
        }

        // Convert to homogeneous coordinates
        let homogeneous_points: Vec<Vec<[f64; 4]>> = control_points
            .iter()
            .zip(weights.iter())
            .map(|(cp_row, w_row)| {
                cp_row
                    .iter()
                    .zip(w_row.iter())
                    .map(|(p, &w)| [p.x * w, p.y * w, p.z * w, w])
                    .collect()
            })
            .collect();

        // Generate uniform knot vectors
        let knots_u = NurbsCurve::generate_uniform_knots(degree_u, num_u);
        let knots_v = NurbsCurve::generate_uniform_knots(degree_v, num_v);

        Ok(Self {
            degree_u,
            degree_v,
            control_points: homogeneous_points,
            knots_u,
            knots_v,
        })
    }

    /// Evaluate the surface at parameters (u, v)
    pub fn evaluate(&self, u: f64, v: f64) -> Result<Point3, NurbsError> {
        if u < self.knots_u[0] || u > *self.knots_u.last().unwrap() ||
           v < self.knots_v[0] || v > *self.knots_v.last().unwrap() {
            return Err(NurbsError::ParameterOutOfRange);
        }

        let nu = self.control_points.len();
        let nv = self.control_points[0].len();

        // Find knot spans
        let span_u = self.find_span_u(u);
        let span_v = self.find_span_v(v);

        // Compute basis functions
        let basis_u = self.basis_functions_u(span_u, u);
        let basis_v = self.basis_functions_v(span_v, v);

        // Compute surface point in homogeneous coordinates
        let mut point = [0.0, 0.0, 0.0, 0.0];

        for i in 0..=self.degree_u {
            for j in 0..=self.degree_v {
                let idx_u = span_u - self.degree_u + i;
                let idx_v = span_v - self.degree_v + j;

                if idx_u < nu && idx_v < nv {
                    let b = basis_u[i] * basis_v[j];
                    point[0] += b * self.control_points[idx_u][idx_v][0];
                    point[1] += b * self.control_points[idx_u][idx_v][1];
                    point[2] += b * self.control_points[idx_u][idx_v][2];
                    point[3] += b * self.control_points[idx_u][idx_v][3];
                }
            }
        }

        // Convert from homogeneous to Cartesian
        if point[3].abs() < EPSILON {
            return Err(NurbsError::DegeneratePoint);
        }

        Ok(Point3::new(
            point[0] / point[3],
            point[1] / point[3],
            point[2] / point[3],
        ))
    }

    /// Find knot span in U direction
    fn find_span_u(&self, u: f64) -> usize {
        self.find_span(u, &self.knots_u, self.control_points.len(), self.degree_u)
    }

    /// Find knot span in V direction
    fn find_span_v(&self, v: f64) -> usize {
        self.find_span(v, &self.knots_v, self.control_points[0].len(), self.degree_v)
    }

    fn find_span(&self, t: f64, knots: &[f64], n: usize, p: usize) -> usize {
        if t >= knots[n] {
            return n - 1;
        }

        let mut low = p;
        let mut high = n;

        while low < high {
            let mid = (low + high) / 2;
            if t < knots[mid] {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        low - 1
    }

    /// Compute basis functions in U direction
    fn basis_functions_u(&self, span: usize, u: f64) -> Vec<f64> {
        self.basis_functions(span, u, self.degree_u, &self.knots_u)
    }

    /// Compute basis functions in V direction
    fn basis_functions_v(&self, span: usize, v: f64) -> Vec<f64> {
        self.basis_functions(span, v, self.degree_v, &self.knots_v)
    }

    fn basis_functions(&self, span: usize, t: f64, p: usize, knots: &[f64]) -> Vec<f64> {
        let mut basis = vec![0.0; p + 1];
        let mut left = vec![0.0; p + 1];
        let mut right = vec![0.0; p + 1];

        basis[0] = 1.0;

        for j in 1..=p {
            left[j] = t - knots[span + 1 - j];
            right[j] = knots[span + j] - t;
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

    /// Tessellate the surface into a triangle mesh
    pub fn tessellate(&self, samples_u: usize, samples_v: usize) -> Result<SurfaceMesh, NurbsError> {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut triangles = Vec::new();

        let u_min = self.knots_u[0];
        let u_max = *self.knots_u.last().unwrap();
        let v_min = self.knots_v[0];
        let v_max = *self.knots_v.last().unwrap();

        // Generate vertices
        for i in 0..samples_u {
            for j in 0..samples_v {
                let u = u_min + (u_max - u_min) * (i as f64 / (samples_u - 1) as f64);
                let v = v_min + (v_max - v_min) * (j as f64 / (samples_v - 1) as f64);

                let point = self.evaluate(u, v)?;
                let normal = self.compute_normal(u, v)?;

                vertices.push(point);
                normals.push(normal);
                uvs.push(Point2::new(
                    i as f64 / (samples_u - 1) as f64,
                    j as f64 / (samples_v - 1) as f64,
                ));
            }
        }

        // Generate triangles
        for i in 0..samples_u - 1 {
            for j in 0..samples_v - 1 {
                let v0 = i * samples_v + j;
                let v1 = i * samples_v + j + 1;
                let v2 = (i + 1) * samples_v + j;
                let v3 = (i + 1) * samples_v + j + 1;

                triangles.push([v0, v1, v2]);
                triangles.push([v1, v3, v2]);
            }
        }

        Ok(SurfaceMesh {
            vertices,
            normals,
            uvs,
            triangles,
        })
    }

    /// Compute surface normal at (u, v)
    fn compute_normal(&self, u: f64, v: f64) -> Result<Vector3, NurbsError> {
        let du = 1e-6;
        let dv = 1e-6;

        let p = self.evaluate(u, v)?;
        let pu = self.evaluate(u + du, v)?;
        let pv = self.evaluate(u, v + dv)?;

        let tangent_u = (pu - p) / du;
        let tangent_v = (pv - p) / dv;

        let normal = tangent_u.cross(&tangent_v);
        let len = normal.norm();

        if len < EPSILON {
            Ok(Vector3::new(0.0, 0.0, 1.0))
        } else {
            Ok(normal / len)
        }
    }
}

/// Tessellated surface mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMesh {
    pub vertices: Vec<Point3>,
    pub normals: Vec<Vector3>,
    pub uvs: Vec<Point2>,
    pub triangles: Vec<[usize; 3]>,
}

/// NURBS-related errors
#[derive(Debug, thiserror::Error)]
pub enum NurbsError {
    #[error("Control points and weights count mismatch")]
    MismatchedControlPointsAndWeights,

    #[error("Insufficient control points for the given degree")]
    InsufficientControlPoints,

    #[error("Invalid knot vector")]
    InvalidKnotVector,

    #[error("Parameter out of valid range")]
    ParameterOutOfRange,

    #[error("Degenerate point (zero weight)")]
    DegeneratePoint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_curve_creation() {
        let control_points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];

        let curve = NurbsCurve::new(2, control_points, weights);
        assert!(curve.is_ok());
    }

    #[test]
    fn test_nurbs_curve_evaluation() {
        let control_points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];

        let curve = NurbsCurve::new(2, control_points, weights).unwrap();
        let point = curve.evaluate(0.5);
        assert!(point.is_ok());
    }

    #[test]
    fn test_nurbs_surface_creation() {
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
        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let surface = NurbsSurface::new(1, 1, control_points, weights);
        assert!(surface.is_ok());
    }
}
