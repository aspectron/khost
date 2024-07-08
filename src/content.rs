use crate::imports::*;

pub enum Content {
    Field(String, String),
    Space,
    Separator,
}

impl Content {
    pub fn field<S1, S2>(k: S1, v: S2) -> Self
    where
        S1: Display,
        S2: Display,
    {
        Content::Field(k.to_string(), v.to_string())
    }

    pub fn space() -> Self {
        Content::Space
    }

    pub fn separator() -> Self {
        Content::Separator
    }

    pub fn length(&self) -> (usize, usize) {
        match self {
            Content::Field(k, v) => (k.len(), v.len()),
            _ => (0, 0),
        }
    }
}

impl From<(&'static str, String)> for Content {
    fn from((k, v): (&'static str, String)) -> Self {
        Content::Field(k.to_string(), v)
    }
}

impl From<(String, String)> for Content {
    fn from((k, v): (String, String)) -> Self {
        Content::Field(k, v)
    }
}

pub fn content(lines: Vec<Content>) -> String {
    let (title_len, content_len) = lines.iter().fold((0, 0), |(a, b), c| {
        let (a_, b_) = c.length();
        (a_.max(a), b_.max(b))
    });

    lines
        .iter()
        .map(|content| match content {
            Content::Field(k, v) => format!(
                "{} {v}",
                k.to_string()
                    .pad_to_width_with_alignment(title_len, Alignment::Right)
            ),
            Content::Space => "".to_string(),
            Content::Separator => "─".repeat(title_len + content_len + 1),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
