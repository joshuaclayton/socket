use super::context::Selector;

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute<'a> {
    Id(&'a str),
    Class(&'a str),
    Custom(&'a str, Vec<AttributeValueComponent<'a>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValueComponent<'a> {
    RawValue(&'a str),
    InterpolatedValue(Vec<Selector<'a>>),
}
