//! # Spatial Indexing for CAD Data
//!
//! Provides high-performance spatial indexing using R-tree and Octree data structures
//! for efficient geometric queries in CAD workloads.

use crate::core::primitives::{BoundingBox2, BoundingBox3, Point2, Point3};
use crate::database::{DatabaseError, Result};
use parking_lot::RwLock;
use rstar::{RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Bounding volume trait for spatial indexing
pub trait BoundingVolume: Send + Sync + Clone {
    /// Get the minimum point of the bounding volume
    fn min(&self) -> Point3;

    /// Get the maximum point of the bounding volume
    fn max(&self) -> Point3;

    /// Check if this volume intersects another
    fn intersects(&self, other: &Self) -> bool;

    /// Check if this volume contains a point
    fn contains_point(&self, point: &Point3) -> bool;

    /// Get the volume size
    fn volume(&self) -> f64;
}

/// Spatial entity for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialEntity {
    /// Entity ID
    pub id: u64,

    /// Bounding box (min and max points)
    pub bbox: SpatialBBox,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Spatial bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpatialBBox {
    /// Minimum point
    pub min: [f64; 3],

    /// Maximum point
    pub max: [f64; 3],
}

impl SpatialBBox {
    /// Create a new 2D bounding box
    pub fn new_2d(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min: [min_x, min_y, 0.0],
            max: [max_x, max_y, 0.0],
        }
    }

    /// Create a new 3D bounding box
    pub fn new_3d(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min: [min_x, min_y, min_z],
            max: [max_x, max_y, max_z],
        }
    }

    /// From BoundingBox2
    pub fn from_bbox2(bbox: &BoundingBox2) -> Self {
        Self::new_2d(bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y)
    }

    /// From BoundingBox3
    pub fn from_bbox3(bbox: &BoundingBox3) -> Self {
        Self::new_3d(
            bbox.min.x, bbox.min.y, bbox.min.z,
            bbox.max.x, bbox.max.y, bbox.max.z,
        )
    }

    /// Check if this box intersects another
    pub fn intersects(&self, other: &Self) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1] && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }

    /// Check if this box contains a point
    pub fn contains(&self, point: [f64; 3]) -> bool {
        point[0] >= self.min[0] && point[0] <= self.max[0]
            && point[1] >= self.min[1] && point[1] <= self.max[1]
            && point[2] >= self.min[2] && point[2] <= self.max[2]
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        (self.max[0] - self.min[0])
            * (self.max[1] - self.min[1])
            * (self.max[2] - self.min[2])
    }

    /// Calculate surface area
    pub fn surface_area(&self) -> f64 {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        2.0 * (dx * dy + dy * dz + dz * dx)
    }
}

// Implement RTreeObject for R-tree indexing
impl RTreeObject for SpatialEntity {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(self.bbox.min, self.bbox.max)
    }
}

/// R-tree spatial index
pub struct RTreeIndex {
    /// R-tree data structure
    tree: Arc<RwLock<RTree<SpatialEntity>>>,

    /// Index statistics
    stats: Arc<RwLock<IndexStats>>,
}

impl RTreeIndex {
    /// Create a new R-tree index
    pub fn new() -> Self {
        Self {
            tree: Arc::new(RwLock::new(RTree::new())),
            stats: Arc::new(RwLock::new(IndexStats::default())),
        }
    }

    /// Insert an entity into the index
    pub fn insert(&self, entity: SpatialEntity) -> Result<()> {
        self.tree.write().insert(entity);
        self.stats.write().total_entities += 1;
        Ok(())
    }

    /// Remove an entity from the index
    pub fn remove(&self, entity: &SpatialEntity) -> Result<bool> {
        let removed = self.tree.write().remove(entity).is_some();
        if removed {
            self.stats.write().total_entities -= 1;
        }
        Ok(removed)
    }

    /// Query entities within a bounding box
    pub fn query_bbox(&self, bbox: &SpatialBBox) -> Result<Vec<SpatialEntity>> {
        let aabb = AABB::from_corners(bbox.min, bbox.max);
        let results: Vec<SpatialEntity> = self
            .tree
            .read()
            .locate_in_envelope_intersecting(&aabb)
            .cloned()
            .collect();

        self.stats.write().query_count += 1;
        Ok(results)
    }

    /// Query entities that intersect with a point
    pub fn query_point(&self, point: [f64; 3]) -> Result<Vec<SpatialEntity>> {
        let results: Vec<SpatialEntity> = self
            .tree
            .read()
            .locate_at_point(&point)
            .cloned()
            .collect();

        self.stats.write().query_count += 1;
        Ok(results)
    }

    /// Nearest neighbor search
    pub fn nearest(&self, point: [f64; 3], max_count: usize) -> Result<Vec<SpatialEntity>> {
        let results: Vec<SpatialEntity> = self
            .tree
            .read()
            .nearest_neighbor_iter(&point)
            .take(max_count)
            .cloned()
            .collect();

        self.stats.write().query_count += 1;
        Ok(results)
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        self.stats.read().clone()
    }

    /// Clear the index
    pub fn clear(&self) {
        self.tree.write().clear();
        self.stats.write().total_entities = 0;
    }

    /// Get the number of entities in the index
    pub fn len(&self) -> usize {
        self.tree.read().size()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for RTreeIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Octree node for 3D spatial indexing
#[derive(Debug, Clone)]
struct OctreeNode {
    /// Bounding box of this node
    bbox: SpatialBBox,

    /// Entities in this node (if leaf)
    entities: Vec<SpatialEntity>,

    /// Child nodes (if not leaf)
    children: Option<Box<[OctreeNode; 8]>>,

    /// Maximum depth
    max_depth: usize,

    /// Current depth
    depth: usize,

    /// Maximum entities per node before subdivision
    max_entities: usize,
}

impl OctreeNode {
    /// Create a new octree node
    fn new(bbox: SpatialBBox, max_depth: usize, max_entities: usize, depth: usize) -> Self {
        Self {
            bbox,
            entities: Vec::new(),
            children: None,
            max_depth,
            depth,
            max_entities,
        }
    }

    /// Insert an entity
    fn insert(&mut self, entity: SpatialEntity) {
        // If this is a leaf and we haven't reached max entities, just add it
        if self.children.is_none() && self.entities.len() < self.max_entities {
            self.entities.push(entity);
            return;
        }

        // If we've reached max depth, force add to this node
        if self.depth >= self.max_depth {
            self.entities.push(entity);
            return;
        }

        // Subdivide if necessary
        if self.children.is_none() {
            self.subdivide();
        }

        // Find the appropriate child and insert
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bbox.intersects(&entity.bbox) {
                    child.insert(entity.clone());
                }
            }
        }
    }

    /// Subdivide this node into 8 children
    fn subdivide(&mut self) {
        let mid_x = (self.bbox.min[0] + self.bbox.max[0]) / 2.0;
        let mid_y = (self.bbox.min[1] + self.bbox.max[1]) / 2.0;
        let mid_z = (self.bbox.min[2] + self.bbox.max[2]) / 2.0;

        let child_depth = self.depth + 1;

        self.children = Some(Box::new([
            // Bottom layer
            OctreeNode::new(
                SpatialBBox::new_3d(
                    self.bbox.min[0], self.bbox.min[1], self.bbox.min[2],
                    mid_x, mid_y, mid_z,
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    mid_x, self.bbox.min[1], self.bbox.min[2],
                    self.bbox.max[0], mid_y, mid_z,
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    self.bbox.min[0], mid_y, self.bbox.min[2],
                    mid_x, self.bbox.max[1], mid_z,
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    mid_x, mid_y, self.bbox.min[2],
                    self.bbox.max[0], self.bbox.max[1], mid_z,
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            // Top layer
            OctreeNode::new(
                SpatialBBox::new_3d(
                    self.bbox.min[0], self.bbox.min[1], mid_z,
                    mid_x, mid_y, self.bbox.max[2],
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    mid_x, self.bbox.min[1], mid_z,
                    self.bbox.max[0], mid_y, self.bbox.max[2],
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    self.bbox.min[0], mid_y, mid_z,
                    mid_x, self.bbox.max[1], self.bbox.max[2],
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
            OctreeNode::new(
                SpatialBBox::new_3d(
                    mid_x, mid_y, mid_z,
                    self.bbox.max[0], self.bbox.max[1], self.bbox.max[2],
                ),
                self.max_depth,
                self.max_entities,
                child_depth,
            ),
        ]));

        // Move existing entities to children
        let entities = std::mem::take(&mut self.entities);
        for entity in entities {
            self.insert(entity);
        }
    }

    /// Query entities within a bounding box
    fn query_bbox(&self, bbox: &SpatialBBox, results: &mut Vec<SpatialEntity>) {
        // Check if this node's bbox intersects the query bbox
        if !self.bbox.intersects(bbox) {
            return;
        }

        // Add entities from this node
        for entity in &self.entities {
            if entity.bbox.intersects(bbox) {
                results.push(entity.clone());
            }
        }

        // Recurse into children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_bbox(bbox, results);
            }
        }
    }
}

/// Octree spatial index for 3D data
pub struct OctreeIndex {
    /// Root node
    root: Arc<RwLock<OctreeNode>>,

    /// Index statistics
    stats: Arc<RwLock<IndexStats>>,

    /// Configuration
    config: OctreeConfig,
}

/// Octree configuration
#[derive(Debug, Clone)]
pub struct OctreeConfig {
    /// Maximum depth
    pub max_depth: usize,

    /// Maximum entities per node before subdivision
    pub max_entities_per_node: usize,

    /// Initial bounding box
    pub initial_bbox: SpatialBBox,
}

impl Default for OctreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 8,
            max_entities_per_node: 8,
            initial_bbox: SpatialBBox::new_3d(-1000.0, -1000.0, -1000.0, 1000.0, 1000.0, 1000.0),
        }
    }
}

impl OctreeIndex {
    /// Create a new octree index
    pub fn new(config: OctreeConfig) -> Self {
        let root = OctreeNode::new(
            config.initial_bbox,
            config.max_depth,
            config.max_entities_per_node,
            0,
        );

        Self {
            root: Arc::new(RwLock::new(root)),
            stats: Arc::new(RwLock::new(IndexStats::default())),
            config,
        }
    }

    /// Insert an entity
    pub fn insert(&self, entity: SpatialEntity) -> Result<()> {
        self.root.write().insert(entity);
        self.stats.write().total_entities += 1;
        Ok(())
    }

    /// Query entities within a bounding box
    pub fn query_bbox(&self, bbox: &SpatialBBox) -> Result<Vec<SpatialEntity>> {
        let mut results = Vec::new();
        self.root.read().query_bbox(bbox, &mut results);
        self.stats.write().query_count += 1;
        Ok(results)
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        self.stats.read().clone()
    }
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Total number of entities
    pub total_entities: u64,

    /// Total number of queries
    pub query_count: u64,

    /// Average query time in microseconds
    pub avg_query_time_us: u64,
}

/// Spatial index wrapper supporting both R-tree and Octree
pub struct SpatialIndex {
    /// R-tree index for general spatial queries
    rtree: RTreeIndex,

    /// Octree index for 3D spatial queries
    octree: OctreeIndex,

    /// Active index type
    active_type: IndexType,
}

/// Index type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// R-tree (default, best for 2D/3D mixed data)
    RTree,

    /// Octree (best for 3D volumetric data)
    Octree,
}

impl SpatialIndex {
    /// Create a new spatial index
    pub fn new() -> Self {
        Self::with_type(IndexType::RTree)
    }

    /// Create a spatial index with a specific type
    pub fn with_type(index_type: IndexType) -> Self {
        Self {
            rtree: RTreeIndex::new(),
            octree: OctreeIndex::new(OctreeConfig::default()),
            active_type: index_type,
        }
    }

    /// Insert an entity
    pub fn insert(&self, entity: SpatialEntity) -> Result<()> {
        match self.active_type {
            IndexType::RTree => self.rtree.insert(entity),
            IndexType::Octree => self.octree.insert(entity),
        }
    }

    /// Query entities within a bounding box
    pub fn query_bbox(&self, bbox: &SpatialBBox) -> Result<Vec<SpatialEntity>> {
        match self.active_type {
            IndexType::RTree => self.rtree.query_bbox(bbox),
            IndexType::Octree => self.octree.query_bbox(bbox),
        }
    }

    /// Query entities at a point
    pub fn query_point(&self, point: [f64; 3]) -> Result<Vec<SpatialEntity>> {
        match self.active_type {
            IndexType::RTree => self.rtree.query_point(point),
            IndexType::Octree => {
                // For octree, create a small bbox around the point
                let epsilon = 0.0001;
                let bbox = SpatialBBox::new_3d(
                    point[0] - epsilon, point[1] - epsilon, point[2] - epsilon,
                    point[0] + epsilon, point[1] + epsilon, point[2] + epsilon,
                );
                self.octree.query_bbox(&bbox)
            }
        }
    }

    /// Nearest neighbor search (only available for R-tree)
    pub fn nearest(&self, point: [f64; 3], max_count: usize) -> Result<Vec<SpatialEntity>> {
        match self.active_type {
            IndexType::RTree => self.rtree.nearest(point, max_count),
            IndexType::Octree => Err(DatabaseError::SpatialIndex(
                "Nearest neighbor search not supported for Octree".to_string(),
            )),
        }
    }

    /// Get combined statistics
    pub fn stats(&self) -> IndexStats {
        match self.active_type {
            IndexType::RTree => self.rtree.stats(),
            IndexType::Octree => self.octree.stats(),
        }
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtree_index() {
        let index = RTreeIndex::new();

        let entity = SpatialEntity {
            id: 1,
            bbox: SpatialBBox::new_2d(0.0, 0.0, 10.0, 10.0),
            metadata: HashMap::new(),
        };

        assert!(index.insert(entity).is_ok());
        assert_eq!(index.len(), 1);

        let results = index.query_bbox(&SpatialBBox::new_2d(5.0, 5.0, 15.0, 15.0)).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_octree_index() {
        let config = OctreeConfig::default();
        let index = OctreeIndex::new(config);

        let entity = SpatialEntity {
            id: 1,
            bbox: SpatialBBox::new_3d(0.0, 0.0, 0.0, 10.0, 10.0, 10.0),
            metadata: HashMap::new(),
        };

        assert!(index.insert(entity).is_ok());

        let results = index.query_bbox(&SpatialBBox::new_3d(5.0, 5.0, 5.0, 15.0, 15.0, 15.0)).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_bbox_intersection() {
        let bbox1 = SpatialBBox::new_2d(0.0, 0.0, 10.0, 10.0);
        let bbox2 = SpatialBBox::new_2d(5.0, 5.0, 15.0, 15.0);
        let bbox3 = SpatialBBox::new_2d(20.0, 20.0, 30.0, 30.0);

        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
    }
}
