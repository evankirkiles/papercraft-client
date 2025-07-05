use cgmath::num_traits::{self, Num};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// Dimensions for screen rects, like the viewport and select box areas
#[derive(Debug, Default, Clone, Copy, Tsify, Serialize, Deserialize)]
pub struct Dimensions<T> {
    pub width: T,
    pub height: T,
}

impl<T> From<Dimensions<T>> for [T; 2] {
    fn from(val: Dimensions<T>) -> Self {
        [val.width, val.height]
    }
}

impl From<Dimensions<f32>> for Dimensions<u32> {
    fn from(value: Dimensions<f32>) -> Self {
        Self { width: value.width as u32, height: value.height as u32 }
    }
}

impl<T> std::ops::Mul<Dimensions<T>> for Dimensions<T>
where
    T: num_traits::real::Real,
{
    type Output = Dimensions<T>;

    fn mul(self, rhs: Dimensions<T>) -> Dimensions<T> {
        Self { width: self.width * rhs.width, height: self.height * rhs.height }
    }
}

/// An exact location on the screen, in pixels
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Tsify, Serialize, Deserialize)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T> Rect<T>
where
    T: Copy + PartialOrd + Num,
{
    /// Determines whether a point lies inside of the rect or not
    pub fn contains(&self, point: &cgmath::Point2<T>) -> bool {
        let (x_range, y_range) =
            ((self.x..(self.x + self.width)), (self.y..(self.y + self.height)));
        x_range.contains(&point.x) && y_range.contains(&point.y)
    }

    /// Determines whether this rect entirely encloses the other rect
    pub fn contains_rect(&self, other: &Rect<T>) -> bool {
        let (x_range, y_range) =
            ((self.x..(self.x + self.width)), (self.y..(self.y + self.height)));
        x_range.contains(&other.x)
            && y_range.contains(&other.y)
            && x_range.contains(&(other.x + other.width))
            && y_range.contains(&(other.y + other.height))
    }

    pub fn split(&self, ratio: T, vertical: bool) -> (Self, Self) {
        if vertical {
            let height = self.height * ratio;
            (
                Self { height, ..*self },
                Self { height: self.height - height, y: self.y + height, ..*self },
            )
        } else {
            let width = self.width * ratio;
            (
                Self { width, ..*self },
                Self { width: self.width - width, x: self.x + width, ..*self },
            )
        }
    }

    /// Maps a point in NDC into a position within this rect
    pub fn ndc(&self, ndc: cgmath::Point2<T>) -> cgmath::Point2<T> {
        let two = T::one() + T::one();
        let half = T::one() / two;
        cgmath::Point2::new(
            self.x + self.width * (half + ndc.x / two),
            self.y + self.height * (half + ndc.y / two),
        )
    }

    /// Does this rect actually have area?
    pub fn has_area(&self) -> bool {
        self.width != T::zero() && self.height != T::zero()
    }
}

impl<T> Rect<T>
where
    T: num_traits::real::Real,
{
    // Creates a Rect between the provided points
    pub fn between(first: cgmath::Point2<T>, second: cgmath::Point2<T>) -> Self {
        let (width, height) = ((first.x - second.x).abs(), (first.y - second.y).abs());
        Self { x: first.x.min(second.x), y: first.y.min(second.y), width, height }
    }
}

impl From<Rect<f32>> for Rect<u32> {
    fn from(value: Rect<f32>) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}

impl From<Rect<u32>> for Rect<f32> {
    fn from(value: Rect<u32>) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
            width: value.width as f32,
            height: value.height as f32,
        }
    }
}

impl<T> From<Dimensions<T>> for Rect<T>
where
    T: Num,
{
    fn from(value: Dimensions<T>) -> Self {
        Self { x: T::zero(), y: T::zero(), width: value.width, height: value.height }
    }
}

impl<T> From<Rect<T>> for Dimensions<T>
where
    T: Num,
{
    fn from(value: Rect<T>) -> Self {
        Self { width: value.width, height: value.height }
    }
}
