use crate::imp::config::CONFIG;
use anyhow::{anyhow, bail};
use anyhow::{Context, Result};
use std::process::Command;

pub fn open_browser(url: &str) -> Result<()> {
    let args = match &CONFIG.doc.browser {
        Some(args) => args,
        None => bail!("browser not specified; please specify the browser path in your config"),
    };

    let mut args = args.iter();
    let browser = args
        .next()
        .ok_or_else(|| anyhow!("the browser command is empty"))?;
    let args = args.map(|x| if x == "%URL%" { url } else { x });

    Command::new(browser)
        .args(args)
        .spawn()
        .context("failed to spawn the browser")?;

    Ok(())
}
