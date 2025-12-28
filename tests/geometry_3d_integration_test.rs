//! Integration tests for 3D geometry module
//!
//! This file tests the complete 3D geometry system including solids, surfaces,
//! meshes, boolean operations, and extrusions.

use caddy::geometry::*;
use nalgebra::{Point2, Point3, Vector3};
use approx::assert_relative_eq;

#[test]
fn test_solid_primitives() {
    // Test Box3D
    let bbox = Box3D::new(Point3::origin(), 10.0, 10.0, 10.0);
    assert_relative_eq!(bbox.volume(), 1000.0, epsilon = 1e-10);
    assert!(bbox.contains_point(&Point3::new(2.0, 2.0, 2.0)));
    assert!(!bbox.contains_point(&Point3::new(10.0, 0.0, 0.0)));

    // Test Sphere3D
    let sphere = Sphere3D::new(Point3::origin(), 5.0);
    assert_relative_eq!(
        sphere.volume(),
        (4.0 / 3.0) * std::f64::consts::PI * 125.0,
        epsilon = 1e-10
    );
    assert!(sphere.contains_point(&Point3::new(3.0, 0.0, 0.0)));

    // Test Cylinder3D
    let cylinder = Cylinder3D::z_aligned(Point3::origin(), 2.0, 10.0);
    assert_relative_eq!(
        cylinder.volume(),
        std::f64::consts::PI * 4.0 * 10.0,
        epsilon = 1e-10
    );

    // Test Cone3D
    let cone = Cone3D::z_aligned(Point3::origin(), 3.0, 6.0);
    assert!(cone.volume() > 0.0);

    // Test Torus3D
    let torus = Torus3D::xy_plane(Point3::origin(), 10.0, 2.0);
    assert!(torus.volume() > 0.0);

    // Test Wedge3D
    let wedge = Wedge3D::new(Point3::origin(), 4.0, 6.0, 8.0);
    assert_relative_eq!(wedge.volume(), 0.5 * 4.0 * 6.0 * 8.0, epsilon = 1e-10);
}

#[test]
fn test_surfaces() {
    // Test Plane3D
    let plane = Plane3D::xy();
    assert_relative_eq!(
        plane.distance_to_point(&Point3::new(5.0, 3.0, 10.0)),
        10.0,
        epsilon = 1e-10
    );

    let projected = plane.project_point(&Point3::new(5.0, 3.0, 10.0));
    assert_relative_eq!(projected.z, 0.0, epsilon = 1e-10);

    // Test Bezier surface
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
    let p = surface.evaluate(0.5, 0.5);
    assert!(p.x >= 0.0 && p.x <= 1.0);
    assert!(p.y >= 0.0 && p.y <= 1.0);
}

#[test]
fn test_triangle_mesh() {
    let mut mesh = TriangleMesh::new();

    let v0 = mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
    let v1 = mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
    let v2 = mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

    mesh.add_face(TriangleFace::new(v0, v1, v2));

    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.faces.len(), 1);

    // Test normal computation
    mesh.compute_vertex_normals();
    assert!(mesh.vertices[0].normal.is_some());

    // Test subdivision
    let original_faces = mesh.faces.len();
    mesh.subdivide();
    assert_eq!(mesh.faces.len(), original_faces * 4);
}

#[test]
fn test_quad_mesh() {
    let mut mesh = QuadMesh::new();

    mesh.vertices.push(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
    mesh.vertices.push(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
    mesh.vertices.push(Vertex::new(Point3::new(1.0, 1.0, 0.0)));
    mesh.vertices.push(Vertex::new(Point3::new(0.0, 1.0, 0.0)));

    mesh.faces.push(QuadFace::new(0, 1, 2, 3));

    let tri_mesh = mesh.to_triangle_mesh();
    assert_eq!(tri_mesh.faces.len(), 2);
}

#[test]
fn test_csg_operations() {
    let mut mesh1 = TriangleMesh::new();
    mesh1.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
    mesh1.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
    mesh1.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));
    mesh1.add_face(TriangleFace::new(0, 1, 2));

    let mesh2 = mesh1.clone();

    // Test CSG node creation
    let node1 = CSGNode::solid(mesh1.clone());
    let node2 = CSGNode::solid(mesh2.clone());

    let union = CSGNode::union(node1.clone(), node2.clone());
    let subtract = CSGNode::subtract(node1.clone(), node2.clone());
    let intersect = CSGNode::intersect(node1, node2);

    assert!(matches!(union, CSGNode::Operation { .. }));
    assert!(matches!(subtract, CSGNode::Operation { .. }));
    assert!(matches!(intersect, CSGNode::Operation { .. }));

    // Test bounding boxes
    assert!(union.bounding_box().is_some());

    // Test CSG operator
    let operator = CSGOperator::new();
    let result = operator.union(&mesh1, &mesh2);
    assert!(result.vertices.len() > 0);
}

#[test]
fn test_linear_extrusion() {
    let profile = Profile2D::circle(1.0, 8);
    let extruder = LinearExtrude::vertical(5.0);
    let mesh = extruder.extrude(&profile);

    assert!(mesh.vertices.len() > 0);
    assert!(mesh.faces.len() > 0);

    // Should have bottom vertices, top vertices, and faces
    assert_eq!(mesh.vertices.len(), 16); // 8 bottom + 8 top
}

#[test]
fn test_revolution() {
    let profile = Profile2D::new(
        vec![
            Point2::new(2.0, 0.0),
            Point2::new(2.0, 1.0),
            Point2::new(3.0, 1.0),
        ],
        false,
    );

    let revolution = Revolution::full_z(16);
    let mesh = revolution.revolve(&profile);

    assert!(mesh.vertices.len() > 0);
    assert!(mesh.faces.len() > 0);
}

#[test]
fn test_sweep() {
    let profile = Profile2D::circle(0.5, 8);
    let mut path = Path3D::line(Point3::new(0.0, 0.0, 0.0), Point3::new(5.0, 0.0, 0.0));

    let sweep = Sweep::new();
    let mesh = sweep.sweep(&profile, &mut path);

    assert!(mesh.vertices.len() > 0);
    assert!(mesh.faces.len() > 0);
}

#[test]
fn test_sweep_with_twist() {
    let profile = Profile2D::circle(0.5, 8);
    let mut path = Path3D::helix(2.0, 1.0, 3.0, 16);

    let sweep = Sweep::new()
        .with_twist(std::f64::consts::PI * 2.0)
        .with_scale(1.0, 0.5, path.points.len());

    let mesh = sweep.sweep(&profile, &mut path);

    assert!(mesh.vertices.len() > 0);
    assert!(mesh.faces.len() > 0);
}

#[test]
fn test_loft() {
    let profile1 = Profile2D::circle(1.0, 8);
    let profile2 = Profile2D::circle(0.5, 8);
    let profile3 = Profile2D::circle(0.8, 8);

    let loft = Loft::new();
    let mesh = loft.loft(&[profile1, profile2, profile3], &[0.0, 5.0, 10.0]);

    assert!(mesh.vertices.len() > 0);
    assert!(mesh.faces.len() > 0);
}

#[test]
fn test_profile_creation() {
    // Test rectangle
    let rect = Profile2D::rectangle(10.0, 5.0);
    assert_eq!(rect.points.len(), 4);
    assert!(rect.closed);
    assert_relative_eq!(rect.area(), 50.0, epsilon = 1e-10);

    // Test circle
    let circle = Profile2D::circle(5.0, 16);
    assert_eq!(circle.points.len(), 16);
    assert!(circle.closed);

    // Test ellipse
    let ellipse = Profile2D::ellipse(3.0, 2.0, 16);
    assert_eq!(ellipse.points.len(), 16);

    // Test regular polygon
    let hexagon = Profile2D::regular_polygon(5.0, 6);
    assert_eq!(hexagon.points.len(), 6);
}

#[test]
fn test_path_creation() {
    // Test line
    let line = Path3D::line(Point3::origin(), Point3::new(10.0, 0.0, 0.0));
    assert_eq!(line.points.len(), 2);
    assert_relative_eq!(line.length(), 10.0, epsilon = 1e-10);

    // Test arc
    let arc = Path3D::arc(
        Point3::origin(),
        5.0,
        0.0,
        std::f64::consts::PI,
        16,
    );
    assert_eq!(arc.points.len(), 17);

    // Test helix
    let helix = Path3D::helix(2.0, 1.0, 3.0, 16);
    assert!(helix.points.len() > 16);
}

#[test]
fn test_complete_workflow() {
    // Create two profiles
    let profile1 = Profile2D::rectangle(4.0, 4.0);
    let profile2 = Profile2D::circle(2.0, 16);

    // Extrude both profiles
    let extruder = LinearExtrude::vertical(10.0);
    let mesh1 = extruder.extrude(&profile1);
    let mesh2 = extruder.extrude(&profile2);

    // Perform boolean subtraction
    let csg_operator = CSGOperator::new();
    let result = csg_operator.subtract(&mesh1, &mesh2);

    assert!(result.vertices.len() > 0);
    assert!(result.faces.len() > 0);
}

#[test]
fn test_half_edge_mesh() {
    let mut triangle_mesh = TriangleMesh::new();

    triangle_mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
    triangle_mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
    triangle_mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));
    triangle_mesh.add_face(TriangleFace::new(0, 1, 2));

    let he_mesh = HalfEdgeMesh::from_triangle_mesh(&triangle_mesh);

    assert_eq!(he_mesh.vertices.len(), 3);
    assert_eq!(he_mesh.faces.len(), 1);
    assert_eq!(he_mesh.half_edges.len(), 3);

    // Test vertex valence
    let valence = he_mesh.vertex_valence(0);
    assert!(valence > 0);
}

#[test]
fn test_bounding_box() {
    let bb = BoundingBox::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 10.0),
    );

    assert_eq!(bb.center(), Point3::new(5.0, 5.0, 5.0));
    assert!(bb.contains(&Point3::new(5.0, 5.0, 5.0)));
    assert!(!bb.contains(&Point3::new(15.0, 5.0, 5.0)));

    let bb2 = BoundingBox::new(
        Point3::new(5.0, 5.0, 5.0),
        Point3::new(15.0, 15.0, 15.0),
    );

    assert!(bb.intersects(&bb2));
}

#[test]
fn test_stl_export() {
    let mut mesh = TriangleMesh::new();

    mesh.add_vertex(Vertex::new(Point3::new(0.0, 0.0, 0.0)));
    mesh.add_vertex(Vertex::new(Point3::new(1.0, 0.0, 0.0)));
    mesh.add_vertex(Vertex::new(Point3::new(0.0, 1.0, 0.0)));
    mesh.add_face(TriangleFace::new(0, 1, 2));

    let stl_data = mesh.to_stl_data();
    assert_eq!(stl_data.len(), 1);
}
