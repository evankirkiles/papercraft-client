use cgmath::{InnerSpace, Vector3};

/// An axis-aligned bounding box in 3D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3 {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb3 {
    pub const EMPTY: Self = Self {
        min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x
    }

    /// Grows the box to include the given point.
    pub fn extend(&mut self, p: Vector3<f32>) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Returns the smallest box containing both `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        Self {
            min: Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn size(&self) -> Vector3<f32> {
        if self.is_empty() {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        self.max - self.min
    }

    /// Half the diagonal of the box: the radius of the smallest sphere
    /// guaranteed to contain it, regardless of viewing angle. Used to size
    /// camera zoom/dolly limits so the whole box stays reachable.
    pub fn bounding_radius(&self) -> f32 {
        self.size().magnitude() / 2.0
    }

    /// The 8 corners of the box, in a fixed order shared by [`Self::EDGES`].
    pub fn corners(&self) -> [Vector3<f32>; 8] {
        [
            Vector3::new(self.min.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.max.y, self.max.z),
            Vector3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Corner-index pairs (into [`Self::corners`]) forming the box's 12 edges.
    pub const EDGES: [(u8, u8); 12] = [
        // bottom face
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // top face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // verticals connecting bottom to top
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
}
