use crate::model;

pub struct Context<'a> {
    models: Vec<model::Model<'a, 'a>>,
}
