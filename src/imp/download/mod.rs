pub mod atcoder;
pub mod local;

use self::atcoder::{AtCoder, Contest as AtCoderContest};
use self::local::Local;
use crate::imp::fetch::TestCaseProvider;
use crate::imp::fs;
use anyhow::bail;
use anyhow::{Context, Result};
use itertools::Itertools;
use std::path::PathBuf;
use std::{env, fmt};

pub trait ContestProvider {
    fn site_name(&self) -> &str;
    fn contest_id(&self) -> &str;
    fn url(&self) -> &str;
    fn make_fetchers(&self) -> Result<Fetchers>;
}

pub struct Fetchers {
    pub fetchers: Vec<Fetcher>,
    pub contest_id: String,
}

pub struct Fetcher {
    pub provider: Box<dyn TestCaseProvider>,
    pub problem_name: String,
}

impl fmt::Debug for Fetcher {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        b.debug_struct("Fetcher")
            .field("problem", &self.problem_name)
            .field("fetcher", &self.provider.url())
            .finish()
    }
}

impl Fetchers {
    pub fn prepare_generate(&self) -> Result<()> {
        let root = self.create_dirs().context("failed to create directories")?;

        // adjust current directory
        env::set_current_dir(root)?;

        Ok(())
    }

    /// Create directories and return the root directory
    fn create_dirs(&self) -> Result<PathBuf> {
        let dirnames = self.fetchers.iter().map(|f| &f.problem_name).collect_vec();
        let root = fs::create_dirs(&self.contest_id, &dirnames, true)
            .context("failed to create directories")?;

        Ok(root)
    }
}

#[derive(Debug, Clone)]
pub struct ContestDescriptor {
    pub contest_site: String,
    pub contest_id: String,
}

impl ContestDescriptor {
    pub fn new(contest_site: String, contest_id: String) -> ContestDescriptor {
        ContestDescriptor {
            contest_site,
            contest_id,
        }
    }

    pub fn parse(dsc: &str) -> Result<ContestDescriptor> {
        let (contest_site, contest_id) = {
            let sp: Vec<_> = dsc.splitn(2, ':').collect();
            (sp[0].to_string(), sp[1].to_string())
        };

        Ok(ContestDescriptor::new(contest_site, contest_id))
    }

    pub fn resolve_provider(self) -> Result<Box<dyn ContestProvider>> {
        match &*self.contest_site {
            "atcoder" | "at" => {
                if self.contest_id.starts_with("http") {
                    let contest = AtCoderContest::from_url(self.contest_id.to_string());
                    let provider = AtCoder::new(contest);
                    Ok(Box::new(provider) as _)
                } else {
                    let contest = AtCoderContest::from_contest_id(self.contest_id.to_string())
                        .context("failed to parse contest-id")?;
                    let provider = AtCoder::new(contest);
                    Ok(Box::new(provider) as _)
                }
            }
            "local" => {
                let provider = Local::from_path(self.contest_id);
                Ok(Box::new(provider) as _)
            }
            site => bail!("unknown contest site: `{}`", site),
        }
    }
}
