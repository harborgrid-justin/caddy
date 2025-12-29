//! Geometry analysis and mass properties computation
//!
//! Provides algorithms for computing volume, surface area, center of mass,
//! moments of inertia, curvature, and other geometric properties.

use super::mesh::{HalfEdgeMesh, VertexHandle, FaceHandle, MeshError};
use crate::core::{Point3, Vector3, EPSILON};
use nalgebra::{Matrix3, Vector3 as NVector3};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Mass properties of a solid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassProperties {
    /// Volume of the solid
    pub volume: f64,
    /// Surface area
    pub surface_area: f64,
    /// Center of mass
    pub center_of_mass: Point3,
    /// Inertia tensor (relative to center of mass)
    pub inertia_tensor: [[f64; 3]; 3],
    /// Principal moments of inertia
    pub principal_moments: [f64; 3],
    /// Bounding box min point
    pub bbox_min: Point3,
    /// Bounding box max point
    pub bbox_max: Point3,
}

/// Geometry analyzer
pub struct GeometryAnalyzer;

impl GeometryAnalyzer {
    /// Compute mass properties for a closed mesh
    /// Assumes the mesh is a closed, manifold solid with consistent outward normals
    pub fn compute_mass_properties(mesh: &HalfEdgeMesh, density: f64) -> Result<MassProperties, AnalysisError> {
        if !mesh.is_manifold() {
            return Err(AnalysisError::NonManifoldMesh);
        }

        // Compute volume and center of mass using divergence theorem
        let (volume, centroid) = Self::compute_volume_and_centroid(mesh)?;

        if volume < EPSILON {
            return Err(AnalysisError::ZeroVolume);
        }

        // Compute surface area
        let surface_area = Self::compute_surface_area(mesh)?;

        // Compute inertia tensor
        let inertia_tensor = Self::compute_inertia_tensor(mesh, &centroid, density)?;

        // Compute principal moments (eigenvalues of inertia tensor)
        let principal_moments = Self::compute_principal_moments(&inertia_tensor);

        // Compute bounding box
        let (bbox_min, bbox_max) = Self::compute_bounding_box(mesh)?;

        Ok(MassProperties {
            volume: volume.abs(),
            surface_area,
            center_of_mass: centroid,
            inertia_tensor,
            principal_moments,
            bbox_min,
            bbox_max,
        })
    }

    /// Compute volume and centroid using the divergence theorem
    fn compute_volume_and_centroid(mesh: &HalfEdgeMesh) -> Result<(f64, Point3), AnalysisError> {
        let mut volume = 0.0;
        let mut centroid = Vector3::zeros();

        for fh in mesh.face_handles() {
            let vertices = mesh.face_vertices(fh).map_err(|_| AnalysisError::InvalidMesh)?;

            if vertices.len() < 3 {
                continue;
            }

            // Get first vertex as reference
            let v0 = mesh.get_vertex(vertices[0]).map_err(|_| AnalysisError::InvalidMesh)?;
            let p0 = v0.position;

            // Triangulate the face and sum contributions
            for i in 1..vertices.len() - 1 {
                let v1 = mesh.get_vertex(vertices[i]).map_err(|_| AnalysisError::InvalidMesh)?;
                let v2 = mesh.get_vertex(vertices[i + 1]).map_err(|_| AnalysisError::InvalidMesh)?;

                let p1 = v1.position;
                let p2 = v2.position;

                // Signed volume of tetrahedron formed by origin and triangle
                let tet_volume = Self::tetrahedron_volume(&Point3::origin(), &p0, &p1, &p2);
                volume += tet_volume;

                // Contribution to centroid
                let tet_centroid = (p0.coords + p1.coords + p2.coords) / 4.0;
                centroid += tet_centroid * tet_volume;
            }
        }

        if volume.abs() < EPSILON {
            return Ok((0.0, Point3::origin()));
        }

        centroid /= volume;

        Ok((volume, Point3::from(centroid)))
    }

    /// Compute signed volume of a tetrahedron
    fn tetrahedron_volume(p0: &Point3, p1: &Point3, p2: &Point3, p3: &Point3) -> f64 {
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let v3 = p3 - p0;

        v1.dot(&v2.cross(&v3)) / 6.0
    }

    /// Compute surface area
    fn compute_surface_area(mesh: &HalfEdgeMesh) -> Result<f64, AnalysisError> {
        let mut area = 0.0;

        for fh in mesh.face_handles() {
            let vertices = mesh.face_vertices(fh).map_err(|_| AnalysisError::InvalidMesh)?;

            if vertices.len() < 3 {
                continue;
            }

            // Triangulate and sum areas
            let v0 = mesh.get_vertex(vertices[0]).map_err(|_| AnalysisError::InvalidMesh)?;
            let p0 = v0.position;

            for i in 1..vertices.len() - 1 {
                let v1 = mesh.get_vertex(vertices[i]).map_err(|_| AnalysisError::InvalidMesh)?;
                let v2 = mesh.get_vertex(vertices[i + 1]).map_err(|_| AnalysisError::InvalidMesh)?;

                let p1 = v1.position;
                let p2 = v2.position;

                area += Self::triangle_area(&p0, &p1, &p2);
            }
        }

        Ok(area)
    }

    /// Compute area of a triangle
    fn triangle_area(p0: &Point3, p1: &Point3, p2: &Point3) -> f64 {
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        v1.cross(&v2).norm() / 2.0
    }

    /// Compute inertia tensor
    fn compute_inertia_tensor(
        mesh: &HalfEdgeMesh,
        centroid: &Point3,
        density: f64,
    ) -> Result<[[f64; 3]; 3], AnalysisError> {
        let mut inertia = [[0.0f64; 3]; 3];

        for fh in mesh.face_handles() {
            let vertices = mesh.face_vertices(fh).map_err(|_| AnalysisError::InvalidMesh)?;

            if vertices.len() < 3 {
                continue;
            }

            let v0 = mesh.get_vertex(vertices[0]).map_err(|_| AnalysisError::InvalidMesh)?;
            let p0 = v0.position - centroid;

            for i in 1..vertices.len() - 1 {
                let v1 = mesh.get_vertex(vertices[i]).map_err(|_| AnalysisError::InvalidMesh)?;
                let v2 = mesh.get_vertex(vertices[i + 1]).map_err(|_| AnalysisError::InvalidMesh)?;

                let p1 = v1.position - centroid;
                let p2 = v2.position - centroid;

                // Tetrahedron contribution
                let tet_vol = Self::tetrahedron_volume(&Point3::origin(), &p0, &p1, &p2);

                // Add to inertia tensor
                for j in 0..3 {
                    for k in 0..3 {
                        let contrib = Self::tetrahedron_inertia_component(
                            &Point3::origin(), &p0, &p1, &p2,
                            j, k,
                        );
                        inertia[j][k] += contrib * density * tet_vol.abs();
                    }
                }
            }
        }

        Ok(inertia)
    }

    /// Compute inertia tensor component for a tetrahedron
    fn tetrahedron_inertia_component(
        p0: &Point3,
        p1: &Point3,
        p2: &Point3,
        p3: &Point3,
        i: usize,
        j: usize,
    ) -> f64 {
        // Simplified inertia calculation
        let points = [p0, p1, p2, p3];
        let mut sum = 0.0;

        for p in &points {
            let coords = [p.x, p.y, p.z];
            if i == j {
                // Diagonal terms
                sum += coords.iter().enumerate()
                    .filter(|(idx, _)| *idx != i)
                    .map(|(_, &c)| c * c)
                    .sum::<f64>();
            } else {
                // Off-diagonal terms
                sum -= coords[i] * coords[j];
            }
        }

        sum / 20.0 // Integration factor for tetrahedron
    }

    /// Compute principal moments of inertia (eigenvalues)
    fn compute_principal_moments(inertia: &[[f64; 3]; 3]) -> [f64; 3] {
        // Convert to nalgebra matrix for eigenvalue computation
        let matrix = Matrix3::new(
            inertia[0][0], inertia[0][1], inertia[0][2],
            inertia[1][0], inertia[1][1], inertia[1][2],
            inertia[2][0], inertia[2][1], inertia[2][2],
        );

        // Compute eigenvalues
        if let Some(eigen) = matrix.symmetric_eigen() {
            let eigenvalues = eigen.eigenvalues;
            [eigenvalues[0], eigenvalues[1], eigenvalues[2]]
        } else {
            // Fallback if eigenvalue computation fails
            [inertia[0][0], inertia[1][1], inertia[2][2]]
        }
    }

    /// Compute bounding box
    fn compute_bounding_box(mesh: &HalfEdgeMesh) -> Result<(Point3, Point3), AnalysisError> {
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for vh in mesh.vertex_handles() {
            let vertex = mesh.get_vertex(vh).map_err(|_| AnalysisError::InvalidMesh)?;
            let p = vertex.position;

            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);

            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }

        Ok((min, max))
    }

    /// Compute Gaussian curvature at a vertex
    pub fn compute_gaussian_curvature(mesh: &HalfEdgeMesh, vh: VertexHandle) -> Result<f64, AnalysisError> {
        let vertex = mesh.get_vertex(vh).map_err(|_| AnalysisError::InvalidMesh)?;

        if vertex.halfedge.is_none() {
            return Ok(0.0);
        }

        // Sum of angles around the vertex
        let mut angle_sum = 0.0;
        let mut area_sum = 0.0;

        // Iterate through faces around vertex
        let he_start = vertex.halfedge.unwrap();
        let mut he = he_start;

        loop {
            if let Ok(halfedge) = mesh.get_halfedge(he) {
                if let Some(face_handle) = halfedge.face {
                    // Compute angle at this vertex in this face
                    if let Ok(angle) = Self::compute_vertex_angle_in_face(mesh, vh, face_handle) {
                        angle_sum += angle;
                    }

                    // Compute face area contribution
                    if let Ok(vertices) = mesh.face_vertices(face_handle) {
                        if vertices.len() >= 3 {
                            let v0 = mesh.get_vertex(vertices[0]).map_err(|_| AnalysisError::InvalidMesh)?;
                            let v1 = mesh.get_vertex(vertices[1]).map_err(|_| AnalysisError::InvalidMesh)?;
                            let v2 = mesh.get_vertex(vertices[2]).map_err(|_| AnalysisError::InvalidMesh)?;
                            area_sum += Self::triangle_area(&v0.position, &v1.position, &v2.position) / 3.0;
                        }
                    }
                }

                // Move to next halfedge around vertex
                if let Some(twin) = halfedge.twin {
                    if let Ok(twin_he) = mesh.get_halfedge(twin) {
                        he = twin_he.next;
                        if he == he_start {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if area_sum < EPSILON {
            return Ok(0.0);
        }

        // Gaussian curvature = (2π - angle_sum) / area
        Ok((2.0 * PI - angle_sum) / area_sum)
    }

    /// Compute angle at a vertex within a face
    fn compute_vertex_angle_in_face(
        mesh: &HalfEdgeMesh,
        vh: VertexHandle,
        fh: FaceHandle,
    ) -> Result<f64, AnalysisError> {
        let vertices = mesh.face_vertices(fh).map_err(|_| AnalysisError::InvalidMesh)?;

        // Find the vertex in the face
        let idx = vertices.iter().position(|&v| v == vh)
            .ok_or(AnalysisError::InvalidMesh)?;

        let n = vertices.len();
        let prev_idx = if idx == 0 { n - 1 } else { idx - 1 };
        let next_idx = (idx + 1) % n;

        let v_prev = mesh.get_vertex(vertices[prev_idx]).map_err(|_| AnalysisError::InvalidMesh)?;
        let v_curr = mesh.get_vertex(vertices[idx]).map_err(|_| AnalysisError::InvalidMesh)?;
        let v_next = mesh.get_vertex(vertices[next_idx]).map_err(|_| AnalysisError::InvalidMesh)?;

        let edge1 = (v_prev.position - v_curr.position).normalize();
        let edge2 = (v_next.position - v_curr.position).normalize();

        let dot = edge1.dot(&edge2).clamp(-1.0, 1.0);
        Ok(dot.acos())
    }

    /// Compute mean curvature at a vertex
    pub fn compute_mean_curvature(mesh: &HalfEdgeMesh, vh: VertexHandle) -> Result<f64, AnalysisError> {
        let vertex = mesh.get_vertex(vh).map_err(|_| AnalysisError::InvalidMesh)?;

        if vertex.halfedge.is_none() {
            return Ok(0.0);
        }

        // Simplified mean curvature approximation
        // Use the Laplace-Beltrami operator

        let mut laplacian = Vector3::zeros();
        let mut weight_sum = 0.0;

        let he_start = vertex.halfedge.unwrap();
        let mut he = he_start;

        loop {
            if let Ok(halfedge) = mesh.get_halfedge(he) {
                let neighbor = mesh.get_vertex(halfedge.vertex).map_err(|_| AnalysisError::InvalidMesh)?;
                let edge = neighbor.position - vertex.position;

                // Uniform weighting (can be improved with cotangent weights)
                let weight = 1.0;
                laplacian += edge * weight;
                weight_sum += weight;

                if let Some(twin) = halfedge.twin {
                    if let Ok(twin_he) = mesh.get_halfedge(twin) {
                        he = twin_he.next;
                        if he == he_start {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if weight_sum < EPSILON {
            return Ok(0.0);
        }

        laplacian /= weight_sum;
        let mean_curvature = laplacian.norm() / 2.0;

        Ok(mean_curvature)
    }
}

/// Analysis errors
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Mesh is not manifold")]
    NonManifoldMesh,

    #[error("Volume is zero or negative")]
    ZeroVolume,

    #[error("Invalid mesh structure")]
    InvalidMesh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.0, 1.0, 0.0);

        let area = GeometryAnalyzer::triangle_area(&p0, &p1, &p2);
        assert!((area - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_tetrahedron_volume() {
        let p0 = Point3::new(0.0, 0.0, 0.0);
        let p1 = Point3::new(1.0, 0.0, 0.0);
        let p2 = Point3::new(0.0, 1.0, 0.0);
        let p3 = Point3::new(0.0, 0.0, 1.0);

        let volume = GeometryAnalyzer::tetrahedron_volume(&p0, &p1, &p2, &p3);
        assert!((volume - 1.0 / 6.0).abs() < EPSILON);
    }
}
