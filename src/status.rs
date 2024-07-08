use crate::imports::*;
use crate::system::System;

pub struct Status {
    pub ip: Option<String>,
    pub system: Rc<System>,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<Content> = self.system.as_ref().into();
        rows.push(Content::separator());
        rows.push(Content::field(
            "Public IP:",
            self.ip.as_deref().unwrap_or("N/A"),
        ));
        writeln!(f, "{}", content(rows))?;
        Ok(())
    }
}

pub fn detect(ctx: &Context) -> Status {
    let ip = ip::blocking::public().ok();
    let system = ctx.system.clone();
    Status { ip, system }
}

pub enum Conflict {
    Warning(String),
    Error(String),
}

impl Conflict {
    pub fn warning(msg: impl Display) -> Self {
        Conflict::Warning(msg.to_string())
    }

    pub fn error(msg: impl Display) -> Self {
        Conflict::Error(msg.to_string())
    }

    pub fn render(&self) -> Result<()> {
        match self {
            Conflict::Warning(msg) => log::warning(msg)?,
            Conflict::Error(msg) => log::error(msg)?,
        }
        Ok(())
    }
}

#[allow(clippy::vec_init_then_push)]
pub fn conflicts(_ctx: &Context, _status: &Status) -> Option<Vec<String>> {
    let mut list = vec![];

    list.push("Kaspad is not running".to_string());
    (!list.is_empty()).then_some(list)
}
