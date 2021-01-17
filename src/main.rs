use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use elefren::MastodonClient;
use structopt::StructOpt;

#[derive(serde::Deserialize, getset::Getters, Debug)]
pub struct Conf {
    #[getset(get = "pub")]
    mastodon_data: PathBuf,

    #[getset(get = "pub")]
    repository_path: PathBuf,

    #[getset(get = "pub")]
    master_branch_name: String,

    #[getset(get = "pub")]
    origin_remote_name: String,

    #[getset(get = "pub")]
    hours_to_check: i64,

    #[getset(get = "pub")]
    status_language: String,

    #[getset(get = "pub")]
    status_template: String,
}

#[derive(structopt::StructOpt, getset::Getters, Debug)]
pub struct Opts {
    #[structopt(short, long, parse(from_os_str))]
    #[getset(get = "pub")]
    config: PathBuf,
}


fn main() -> Result<()> {
    env_logger::init();
    log::debug!("Logger initialized");

    let opts = Opts::from_args_safe()?;
    let config: Conf = {
        let mut config = ::config::Config::default();

        config
            .merge(::config::File::from(opts.config().to_path_buf()))?
            .merge(::config::Environment::with_prefix("COMBOT"))?;
        config.try_into()?
    };
    let mastodon_data: elefren::Data = toml::de::from_str(&std::fs::read_to_string(config.mastodon_data())?)?;
    let client = elefren::Mastodon::from(mastodon_data);
    let status_language = elefren::Language::from_639_1(config.status_language())
        .ok_or_else(|| anyhow!("Could not parse status language code: {}", config.status_language()))?;
    log::debug!("config parsed");

    let repo = git2::Repository::open(config.repository_path())?;
    log::debug!("Repo opened successfully");
    let _ = fetch_main_remote(&repo, &config)?;
    log::debug!("Main branch fetched successfully");

    let (commits, merges, nonmerges) = count_commits_on_main_branch(&repo, &config)?;
    log::debug!("Counted commits successfully");

    log::info!("Commits    = {}", commits);
    log::info!("Merges     = {}", merges);
    log::info!("Non-Merges = {}", nonmerges);

    {
        let status_text = {
            let mut hb = handlebars::Handlebars::new();
            hb.register_template_string("status", config.status_template())?;
            let mut data = std::collections::BTreeMap::new();
            data.insert("commits", commits);
            data.insert("merges", merges);
            data.insert("nonmerges", nonmerges);
            hb.render("status", &data)?
        };

        let status = elefren::StatusBuilder::new()
            .status(status_text)
            .language(status_language)
            .build()
            .expect("Failed to build status");

        let status = client.new_status(status)
            .expect("Failed to post status");
        if let Some(url) = status.url.as_ref() {
            log::info!("Status posted: {}", url);
        } else {
            log::info!("Status posted, no url");
        }
        log::debug!("New status = {:?}", status);
    }

    Ok(())
}

fn fetch_main_remote(repo: &git2::Repository, config: &Conf) -> Result<()> {
    log::debug!("Fetch: {} / {}", config.origin_remote_name(), config.master_branch_name());
    repo.find_remote(config.origin_remote_name())?
        .fetch(&[config.master_branch_name()], None, None)
        .map_err(Error::from)
}

fn count_commits_on_main_branch(repo: &git2::Repository, config: &Conf) -> Result<(usize, usize, usize)> {
    let branchname = format!("{}/{}", config.origin_remote_name(), config.master_branch_name());
    let minimum_time_epoch = chrono::offset::Local::now().timestamp() - (config.hours_to_check() * 60 * 60);

    log::debug!("Branch to count     : {}", branchname);
    log::debug!("Earliest commit time: {:?}", minimum_time_epoch);

    let revwalk_start = repo
        .find_branch(&branchname, git2::BranchType::Remote)?
        .get()
        .peel_to_commit()?
        .id();

    log::debug!("Starting at: {}", revwalk_start);

    let mut rw = repo.revwalk()?;
    rw.simplify_first_parent()?;
    rw.push(revwalk_start)?;

    let mut commits = 0;
    let mut merges = 0;
    let mut nonmerges = 0;

    for rev in rw {
        let rev = rev?;
        let commit = repo.find_commit(rev)?;
        log::trace!("Found commit: {:?}", commit);

        if commit.time().seconds() < minimum_time_epoch {
            log::trace!("Commit too old, stopping iteration");
            break;
        }
        commits += 1;

        let is_merge = commit.parent_ids().count() > 1;
        log::trace!("Merge: {:?}", is_merge);

        if is_merge {
            merges += 1;
        } else {
            nonmerges += 1;
        }
    }

    log::trace!("Ready iterating");
    Ok((commits, merges, nonmerges))
}

