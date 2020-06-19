use super::{context::Context, Attribute, AttributeValueComponent};

pub struct Attributes<'a> {
    id: Option<&'a str>,
    classes: Vec<&'a str>,
    custom: Vec<(&'a str, Vec<AttributeValueComponent<'a>>)>,
}

impl<'a> Attributes<'a> {
    pub fn to_html(&self, context: &Context) -> Vec<String> {
        let mut results = vec![];

        if let Some(id) = self.id {
            results.push(format!("id=\"{}\"", id));
        }

        if !self.classes.is_empty() {
            results.push(format!("class=\"{}\"", self.classes.join(" ")));
        }

        for (k, v) in self.custom.iter() {
            results.push(format!(
                "{}=\"{}\"",
                k,
                evaluate_attribute_value_components(&v, context)
            ));
        }

        results
    }
}

fn evaluate_attribute_value_components<'a>(
    values: &[AttributeValueComponent<'a>],
    context: &Context,
) -> String {
    values
        .iter()
        .map(|v| match v {
            AttributeValueComponent::RawValue(value) => value.to_string(),
            AttributeValueComponent::InterpolatedValue(values) => context.interpret(values),
        })
        .collect()
}

impl<'a> From<Vec<Attribute<'a>>> for Attributes<'a> {
    fn from(attributes: Vec<Attribute<'a>>) -> Self {
        let id = attributes
            .iter()
            .filter_map(|a| match *a {
                Attribute::Id(v) => Some(v),
                _ => None,
            })
            .nth(0);

        let classes = attributes
            .iter()
            .filter_map(|a| match *a {
                Attribute::Class(v) => Some(v),
                _ => None,
            })
            .collect();

        let custom = attributes
            .iter()
            .filter_map(|a| match a {
                Attribute::Custom(k, v) => Some((k.clone(), v.clone())),
                _ => None,
            })
            .collect();

        Attributes {
            id,
            classes,
            custom,
        }
    }
}
