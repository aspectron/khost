use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Origin {
    repository: String,
    owner: String,
    name: String,
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
            let repo = repository.replace(".git", "");
            let mut parts = repo.split('/').collect::<VecDeque<_>>();
            let name = parts.pop_back().unwrap().to_string();
            let owner = parts.pop_back().unwrap().to_string();

            Ok(Self {
                repository,
                name,
                owner,
                branch: branch.map(String::from),
            })
        }
    }

    pub fn folder(&self) -> PathBuf {
        // let repo = self.repository.replace(".git", "");
        // let mut parts = repo.split('/').collect::<VecDeque<_>>();
        // let _ = parts.pop_back().unwrap();
        // let owner = parts.pop_back().unwrap();
        PathBuf::from(format!(
            "{}/{}",
            self.owner,
            self.branch.as_deref().unwrap_or("master")
        ))
    }

    // pub fn folder(&self) -> PathBuf {
    //     let repo = self.repository.replace(".git", "");
    //     let mut parts = repo.split('/').collect::<VecDeque<_>>();
    //     let _ = parts.pop_back().unwrap();
    //     let owner = parts.pop_back().unwrap();
    //     PathBuf::from(format!(
    //         "{}/{}",
    //         org,
    //         self.branch.as_deref().unwrap_or("master")
    //     ))
    // }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn api_url(&self) -> String {
        let Origin {
            owner,
            name,
            branch,
            ..
        } = self;

        format!(
            "https://api.github.com/repos/{}/{}/branches/{}",
            owner,
            name,
            branch.as_deref().unwrap_or("master")
        )
    }

    pub fn name(&self) -> &str {
        &self.name
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

pub fn pull<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["pull"]).dir(path).run()?;

    Ok(())
}

pub fn reset<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["reset", "--hard", "HEAD"]).dir(path).run()?;

    Ok(())
}

pub fn hash<P: AsRef<Path>>(path: P, short: bool) -> Result<String> {
    let path = path.as_ref().display().to_string();

    let hash = duct::cmd("git", &["rev-parse", "HEAD"]).dir(path).read()?;
    let hash = hash.trim();
    if short {
        if hash.len() < 7 {
            Err(Error::Hash(hash.to_string()))
        } else {
            Ok(hash[0..7].to_string())
        }
    } else {
        Ok(hash.trim().to_string())
    }
}

#[derive(Deserialize)]
struct Commit {
    sha: String,
}

#[derive(Deserialize)]
struct Branch {
    commit: Commit,
}

pub fn latest_commit_hash(origin: &Origin, short: bool) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(origin.api_url())
        .header("User-Agent", "khost")
        .send()?
        .json::<Branch>()?;

    let hash = if short {
        response.commit.sha[..7].to_string()
    } else {
        response.commit.sha
    };

    Ok(hash)
}

pub fn version() -> Option<String> {
    cmd!("git", "--version")
        .read()
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn create_origin_impl<S>(name: S, origin: Option<Origin>) -> Result<Origin>
where
    S: Display,
{
    let name = name.to_string();

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    enum Preset {
        Omega,
        Delta,
        Custom,
    }

    let preset = if name == "rusty-kaspa" {
        cliclack::select(format!("Select git origin for '{name}':"))
            .item(Preset::Omega, "Omega (aspectron/omega - wRPC v2)", "")
            .item(
                Preset::Delta,
                "Delta",
                "(aspectron/delta - wRPC v2 + GD perf)",
            )
            .item(Preset::Custom, "Custom", "")
            .interact()?
    } else {
        Preset::Custom
    };

    let origin = match preset {
        Preset::Omega => {
            Origin::try_new("https://github.com/aspectron/rusty-kaspa", Some("omega"))?
        }
        Preset::Delta => {
            Origin::try_new("https://github.com/aspectron/rusty-kaspa", Some("delta"))?
        }
        Preset::Custom => {
            let mut input = cliclack::input("Enter GitHub repository owner/organization:")
                .placeholder("")
                .validate(|v: &String| {
                    if v.trim().is_empty() {
                        Err("Please enter a valid owner/organization name".to_string())
                    } else {
                        Ok(())
                    }
                });
            if let Some(origin) = &origin {
                input = input.default_input(origin.owner.as_str());
            }
            let owner: String = input.interact()?;

            let mut input = cliclack::input("Enter repository branch: ").required(false);
            if let Some(origin) = &origin {
                if let Some(branch) = origin.branch() {
                    input = input.default_input(branch);
                }
            }
            let branch: String = input.interact()?;
            let branch = branch.trim().to_string();
            let branch = (!branch.is_empty()).then_some(branch);

            let origin = Origin::try_new(
                &format!("https://github.com/{owner}/{name}"),
                branch.as_deref(),
            )?;
            latest_commit_hash(&origin, false).map_err(|_| Error::Origin(origin.clone()))?;
            origin
        }
    };

    Ok(origin)
}

pub fn create_origin<S>(name: S) -> Result<Origin>
where
    S: Display,
{
    let mut last_origin = None;
    loop {
        match create_origin_impl(&name, last_origin) {
            Ok(origin) => {
                return Ok(origin);
            }
            Err(Error::Origin(origin)) => {
                last_origin = Some(origin);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
