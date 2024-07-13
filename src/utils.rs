use crate::imports::*;

pub fn terminal_width() -> usize {
    termion::terminal_size()
        .map(|size| size.0 as usize)
        .unwrap_or(80)
}

pub fn truncate_to_terminal<S>(text: S) -> String
where
    S: Display,
{
    let text = text.to_string();
    let terminal_width = terminal_width();
    text.lines()
        .map(|line| {
            let mut line = line.to_string();
            if line.len() > terminal_width {
                line.truncate(terminal_width - 3);
                line.push_str("...");
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}
