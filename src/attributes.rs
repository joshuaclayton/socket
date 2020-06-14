use super::Attribute;

pub struct Attributes<'a> {
    id: Option<&'a str>,
    classes: Vec<&'a str>,
    custom: Vec<(&'a str, &'a str)>,
}

impl<'a> Attributes<'a> {
    pub fn to_html(&self) -> Vec<String> {
        let mut results = vec![];

        if let Some(id) = self.id {
            results.push(format!("id=\"{}\"", id));
        }

        if !self.classes.is_empty() {
            results.push(format!("class=\"{}\"", self.classes.join(" ")));
        }

        for (k, v) in self.custom.iter() {
            results.push(format!("{}=\"{}\"", k, v));
        }

        results
    }
}

impl<'a> From<Vec<Attribute<'a>>> for Attributes<'a> {
    fn from(attributes: Vec<Attribute<'a>>) -> Self {
        let id = attributes
            .iter()
            .filter_map(|&a| match a {
                Attribute::Id(v) => Some(v),
                _ => None,
            })
            .nth(0);

        let classes = attributes
            .iter()
            .filter_map(|&a| match a {
                Attribute::Class(v) => Some(v),
                _ => None,
            })
            .collect();

        let custom = attributes
            .iter()
            .filter_map(|&a| match a {
                Attribute::Custom(k, v) => Some((k, v)),
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
