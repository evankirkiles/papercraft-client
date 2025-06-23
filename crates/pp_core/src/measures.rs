#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalPosition<T> {
    pub x: T,
    pub y: T,
}

impl<T> std::ops::Mul<PhysicalPosition<T>> for PhysicalPosition<T>
where
    T: std::ops::Mul<T, Output = T>,
{
    type Output = PhysicalPosition<T>;

    fn mul(self, rhs: PhysicalPosition<T>) -> PhysicalPosition<T> {
        Self { x: self.x * rhs.x, y: self.y * rhs.y }
    }
}

impl<T> std::ops::Sub<PhysicalPosition<T>> for PhysicalPosition<T>
where
    T: std::ops::Sub<T, Output = T>,
{
    type Output = PhysicalPosition<T>;

    fn sub(self, rhs: PhysicalPosition<T>) -> PhysicalPosition<T> {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl<T> Into<cgmath::Vector2<T>> for PhysicalPosition<T> {
    fn into(self) -> cgmath::Vector2<T> {
        cgmath::Vector2::new(self.x, self.y)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalDimensions<T> {
    pub width: T,
    pub height: T,
}

impl<T> std::ops::Mul<PhysicalDimensions<T>> for PhysicalDimensions<T>
where
    T: std::ops::Mul<T, Output = T>,
{
    type Output = PhysicalDimensions<T>;

    fn mul(self, rhs: PhysicalDimensions<T>) -> PhysicalDimensions<T> {
        Self { width: self.width * rhs.width, height: self.height * rhs.height }
    }
}
