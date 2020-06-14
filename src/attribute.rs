#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Attribute<'a> {
    Id(&'a str),
    Class(&'a str),
    Custom(&'a str, &'a str),
}
