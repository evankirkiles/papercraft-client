use crate::mesh;

pub struct Context<'a> {
    models: Vec<model::Model<'a>>,
}
