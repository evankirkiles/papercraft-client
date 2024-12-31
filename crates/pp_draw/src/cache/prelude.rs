use crate::gpu;

/// A helper trait indicating 1:1 relationships between a CPU struct and the
/// GPU representation struct.
pub trait GPUCache<T> {
    fn new(ctx: &gpu::Context, item: &T) -> Self;
    fn sync(&mut self, ctx: &gpu::Context, item: &T);
}
