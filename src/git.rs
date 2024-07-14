use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Origin {
    repository: String,
    branch: Option<String>,
}

impl Display for Origin {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.folder().display())
    }
}

impl Origin {
    pub fn try_new(repo: &str, branch: Option<&str>) -> Result<Self> {
        let repository = repo.to_string();
        let parts = repository.split('/').collect::<Vec<_>>();
        if parts.len() < 3 {
            Err(Error::Repository(repository))
        } else {
            Ok(Self {
                repository: repo.to_string(),
                branch: branch.map(String::from),
            })
        }
    }

    pub fn folder(&self) -> PathBuf {
        let repo = self.repository.replace(".git", "");
        let mut parts = repo.split('/').collect::<VecDeque<_>>();
        let _ = parts.pop_back().unwrap();
        let org = parts.pop_back().unwrap();
        PathBuf::from(format!(
            "{}/{}",
            org,
            self.branch.as_deref().unwrap_or("master")
        ))
    }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

pub fn clone<P: AsRef<Path>>(path: P, origin: &Origin) -> Result<()> {
    let path = path.as_ref().display().to_string();

    if let Some(branch) = origin.branch() {
        cmd("git", &["clone", "-b", branch, origin.repository(), &path]).run()?;
    } else {
        cmd("git", &["clone", origin.repository(), &path]).run()?;
    }

    Ok(())
}

pub fn pull<P: AsRef<Path>>(path: P, _origin: &Origin) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["pull", &path]).dir(path).run()?;

    Ok(())
}

pub fn restore<P: AsRef<Path>>(path: P, _origin: &Origin) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["restore", &path]).dir(path).run()?;

    Ok(())
}

pub fn version() -> Option<String> {
    cmd!("git", "--version")
        .read()
        .ok()
        .map(|s| s.trim().to_string())
}
