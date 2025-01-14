#![forbid(unsafe_code)]
use std::{
    fs,
    io::{self, BufRead, Write},
    path::Path,
    process::Command,
    time::SystemTime,
};

use chrono::prelude::*;
use clap::Parser;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use slog::{slog_o, Drain};
use tera::{Context, Tera};
use thiserror::Error;

#[macro_use]
extern crate slog_scope;

const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// The potential errors that we could encounter.
#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Invalid Settings: {0}")]
    SettingsError(String),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Could not process Post file: {0}")]
    PostProcessError(String),
    #[error("Invalid tags specified in Post file: {0}")]
    TagError(#[from] serde_json::Error),
    #[error("Tera Error {0}")]
    TeraError(#[from] tera::Error),
    #[error("General Error {0}")]
    General(String),
}

/// Command line and file arguments what set the things to do what you want.
#[derive(Clone, Debug, Deserialize, Parser)]
pub struct Args {
    /// Optional config file. So you don't have to specify them all. (-c, --config | config.toml)
    #[clap(short, long)]
    pub config: Option<String>,
    #[clap(short, long)]
    /// Directory containing the jinja templates (-t, --templates | "./templates")
    #[clap(short, long)]
    pub templates: Option<String>,
    /// Where to write the created files (-o , --output | "./archive")
    #[clap(short, long)]
    pub output: Option<String>,
    /// Where the markdown files are (-s, --source | "./source")
    #[clap(short, long)]
    pub source: Option<String>,
    /// How many files should be considered "recent"? (-r, --recent | 10)
    #[clap(short, long)]
    pub recent: Option<u32>,
    /// The URL for the blog (--url)
    #[clap(short, long)]
    pub url: Option<String>,
    /// Do you have a short URL? If not, reuse the `url` (--short_url)
    #[clap(long)]
    pub short_url: Option<String>,
    /// Order by creation time or post number? (--by_time | False)
    #[clap(long, action=clap::ArgAction::SetTrue)]
    pub by_time: Option<bool>,
    /// Order by file name (--by_name | True)
    #[clap(long, action=clap::ArgAction::SetTrue)]
    pub by_name: Option<bool>,
    #[clap(long, action=clap::ArgAction::SetTrue)]
    pub new: Option<bool>,
}

/// Internal settings.
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Directory containing the jinja templates (-t, --templates | "./templates")
    pub templates: String,
    /// Where to write the created files (-o , --output | "./archive")
    pub output: String,
    /// Where the markdown files are (-s, --source | "./source")
    pub source: String,
    /// How many files should be considered "recent"? (-r, --recent | 10)
    pub recent: u32,
    /// The URL for the blog (--url)
    pub url: String,
    /// Do you have a short URL? If not, reuse the `url` (--short_url)
    pub short_url: Option<String>,
    /// Order by creation time or post number? (--by_time | False)
    pub by_time: bool,
    /// Order by file name (--by_name | True)
    pub by_name: bool,
    /// just create a new post
    pub new: bool,
}

impl Settings {
    /// Create a new Settings from Args backfilling using defaults.
    fn backfill_using(value: Args, defaults: Settings) -> Self {
        Self {
            templates: value.templates.unwrap_or(defaults.templates),
            output: value.output.unwrap_or(defaults.output),
            source: value.source.unwrap_or(defaults.source),
            recent: value.recent.unwrap_or(defaults.recent),
            url: value.url.unwrap_or(defaults.url),
            short_url: value.short_url.or(defaults.short_url),
            by_time: value.by_time.unwrap_or(defaults.by_time),
            by_name: value.by_name.unwrap_or(defaults.by_name),
            new: value.new.unwrap_or(defaults.new),
        }
    }
}

impl From<Args> for Settings {
    fn from(value: Args) -> Self {
        Self::backfill_using(value, Settings::default())
    }
}

impl Settings {
    pub fn new() -> Result<Self, PublishError> {
        let args = Args::parse();
        debug!("∈ args {:?}", &args);
        let filename = args.config.clone().unwrap_or("config.toml".to_owned());
        let file = Path::new(&filename);
        let mut settings: Settings = if !file.exists() {
            args.into()
        } else {
            let buffer = fs::read_to_string(file).map_err(|e| {
                PublishError::SettingsError(format!(
                    "Could not read config from file {:?} {:?}",
                    filename, e
                ))
            })?;
            let file_args: Args = toml::from_str(&buffer).map_err(|e| {
                PublishError::SettingsError(format!(
                    "Could not parse config file {:?} {:?}",
                    filename, e
                ))
            })?;
            Settings::backfill_using(args, file_args.into())
        };
        debug!("∈ settings {:?}", &settings);
        debug!("∈ Returning settings...");
        if !settings.templates.contains("*") {
            debug!("∈ Fixing templates...");
            settings.templates = format!("{}/*", settings.templates);
        }
        Ok(settings)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            templates: "template/*".to_owned(),
            output: "archive".to_owned(),
            source: "source".to_owned(),
            recent: 10,
            url: "https://blog.unitedheroes.net".to_owned(),
            short_url: None,
            by_time: false,
            by_name: true,
            new: false,
        }
    }
}

fn init_logging() -> Result<(), PublishError> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!());
    slog_scope::set_global_logger(logger).cancel_reset();
    slog_stdlog::init().ok();
    Ok(())
}

/// Convert the markdown post into it's component elements
#[derive(Clone, Debug, Default, Serialize)]
struct Post {
    /// The URL to the Post (generated from the num and name)
    pub link: String,
    /// An optional short link to the post
    pub shortlink: Option<String>,
    /// The proper title of the post
    pub title: String,
    /// The number of the post (as determined by the file name for now)
    pub num: u64,
    /// The list of categories for this post.
    pub tags: Vec<String>,
    /// The markdown content of the post
    pub md_body: String,
    /// the HTML content of the post
    pub body: String,
    /// An optional summary (mostly used by RSS.)
    pub summary: Option<String>,
    /// The URL/file safe name for this post
    pub name: String,
    /// The date for the post (default to the atime of the file.)
    pub timestamp: Option<SystemTime>,
    /// Number of seconds since Epoch (used by forms)
    pub date: u64,
}

impl Post {
    /// Construct the post from the file.
    async fn from_file(settings: &Settings, filepath: String) -> Result<Self, PublishError> {
        let mut result = Self::default();
        let handle = fs::File::open(&filepath).unwrap();
        let atime = handle.metadata().unwrap().created()?;
        let file: io::Lines<io::BufReader<fs::File>> = io::BufReader::new(handle).lines();
        debug!("👀 {:?}", &filepath);
        let (num, name) = match Path::new(&filepath).file_stem() {
            Some(v) => match v.to_str() {
                Some(v) => v,
                None => {
                    return Err(PublishError::PostProcessError(format!(
                        "Invalid characters in file name {filepath}"
                    )))
                }
            },
            None => {
                return Err(PublishError::PostProcessError(format!(
                    "Missing stem for filename {filepath}"
                )))
            }
        }
        .split_once("_")
        .unwrap();
        result.name = name.to_owned();
        result.num = num
            .parse::<u64>()
            .map_err(|e| PublishError::PostProcessError(format!("Invalid file name {:?}", e)))?;
        result.link = format!("{}/{:04}", settings.url, result.num);
        if let Some(link) = settings.short_url.clone() {
            result.shortlink = Some(format!("{}/{:04}", link, result.num));
        }
        let mut is_header = true;
        let mut body = Vec::new();

        // Read the post header and extract the interesting bits.
        for line in file {
            if let Ok(line) = line {
                if is_header {
                    if line.is_empty() {
                        continue;
                    }
                    if line.starts_with("===") {
                        is_header = false;
                    }
                    if line.starts_with("[") {
                        result.parse_tags(&line)?;
                    }
                    if line.starts_with("<!-- Date:") {
                        result.parse_date(&line)?;
                    }
                    if line.starts_with("# ") {
                        result.parse_title(&line)?;
                    }
                    if line.starts_with("> ") {
                        result.summary = Some(
                            format!("{} {}", result.summary.unwrap_or_default(), line.trim())
                                .trim()
                                .to_owned(),
                        );
                    }
                } else {
                    body.push(line);
                    // Need to add a newline because rust strips those.
                    body.push("\n".to_owned());
                }
            } else {
                break;
            }
        }
        if result.timestamp.is_none() {
            result.timestamp = Some(atime);
        }
        result.date = result
            .timestamp
            .unwrap_or(SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                PublishError::General(format!("Could not convert timestamp to int {:?}", e))
            })?
            .as_secs();
        result.md_body = body.join("");
        let parser = pulldown_cmark::Parser::new(&result.md_body);
        pulldown_cmark::html::push_html(&mut result.body, parser);
        Ok(result)
    }

    /// Read the tags from a string (this is because serde_json doesn't handle
    /// strings that are a list. This may be due to a long standing security issue
    /// with JSON not dealing with sets well.)
    fn parse_tags(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        trace!("😺 tags: {:?}", &line);
        // Sadly, serde chokes on just a set, so I can't use that.
        // Have to break these apart the manual way.
        self.tags = line
            .to_owned()
            .replace(['[', ']', '"'], "")
            .split(",")
            .map(|v| v.trim().to_owned())
            .collect();
        Ok(self)
    }

    /// Extract a date from a POST header string
    fn parse_date(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        trace!("📅date: {:?}", &line);
        let re = Regex::new(r"<!-- (Date:)? (?<ts>.*) -->").expect("Date Regex altered");

        let date_str = re.replace(line, "$ts");
        self.timestamp = DateTime::parse_from_rfc2822(&date_str)
            .ok()
            .map(DateTime::<Local>::from)
            .map(Into::into);
        Ok(self)
    }

    /// Extract the post title from the header string. (Basically just strip off the leading "# ")
    fn parse_title(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        self.title = line.strip_prefix("# ").unwrap().to_owned();
        Ok(self)
    }

    #[allow(unused)]
    fn to_file(&self, path: &Path) -> Result<String, PublishError> {
        let file_name = format!("{:?}_{}.md", self.num, self.name);
        let destination = path.join(&file_name);
        info!("Writing: {:?}", destination);
        let mut file = fs::File::create(destination.clone())?;
        // First, write the headers
        file.write_all(self.to_string().as_bytes())?;
        Ok(destination
            .as_os_str()
            .to_str()
            .unwrap_or_default()
            .to_owned())
    }
}

/// Serialize the Post into something we can write to a file and (hopefully, read back later.)
impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags = serde_json::json!(self.tags).to_string();
        let date = time_str(&self.timestamp.unwrap_or(<Local>::now().into()));
        writeln!(f, "# {}\n{}\n{}", self.title, date, tags)?;
        if let Some(summary) = self.summary.clone() {
            writeln!(f, "> {}", summary)?;
        }
        writeln!(f, "===\n{}", self.body)?;
        Ok(())
    }
}

/// Return a normalized Time String for things.
fn time_str(time: &SystemTime) -> String {
    DateTime::<Local>::from(*time)
        .format(TIME_FORMAT)
        .to_string()
}

/// Get a list of files based on the leading number. (ideally, this should optionally sort based
/// on the atime of the files like the python version does, but that can wait.
fn get_latest_files(settings: &Settings) -> Result<Vec<String>, PublishError> {
    let mut files: Vec<String> = Vec::new();
    let re = Regex::new(r"^[0-9]{4}.*\.md$").expect("WTF: Regex failed to compile");
    trace!("📁 Reading source... {}", &settings.source);
    for file in fs::read_dir(settings.source.clone()).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let filename = path
            .file_name()
            .unwrap_or_else(|| panic!("WTF: Couldn't get file? {:?}", &file))
            .to_str()
            .unwrap_or_else(|| panic!("WTF: Couldn't make file a string? {:?}", &file));
        if file.path().is_file() && re.is_match(filename) {
            files.push(path.as_os_str().to_str().unwrap().to_owned());
        }
    }
    files.sort();
    let (_left, right) = files
        .split_at_checked(files.len() - settings.recent as usize)
        .unwrap_or((&[], &files));
    Ok(right.to_vec())
}

/// Get a set of the latest files, read them, and return Posts for those files.
async fn get_latest_posts(settings: &Settings) -> Result<Vec<Post>, PublishError> {
    let mut posts = Vec::new();

    for filepath in get_latest_files(settings)? {
        posts.push(Post::from_file(settings, filepath).await?)
    }
    Ok(posts)
}

/// Write a composed Tera file for a given Post.
async fn write_post(
    settings: &Settings,
    tera: &Tera,
    current: &Post,
    prev: Option<&Post>,
    next: Option<&Post>,
) -> Result<Option<String>, PublishError> {
    let path = Path::new(&settings.output).join(format!("{}.php", &current.num));
    debug!("✍️ writing post ✍️ {:?}", &path);
    let mut context = Context::new();
    context.insert("post", &current);
    if let Some(prev) = prev {
        context.insert("prev", &prev);
    }
    if let Some(next) = next {
        context.insert("next", &next);
    }
    context.insert("url", &settings.url);
    context.insert(
        "short_url",
        &settings.short_url.clone().unwrap_or(settings.url.clone()),
    );
    trace!("✍️ Writing ...{} to {:?}", &current.title, &path);
    // Current post file
    if let Some(name) = path.to_str().map(|v| v.to_owned()) {
        let mut current_file = fs::File::create(&path)?;
        trace!("✍️ opened. {:?}", &name);
        let _ = current_file.write_all(tera.render("index.php", &context)?.as_bytes());
        trace!("✍ done {:?}", &name);
        return Ok(Some(name));
    }
    Ok(None)
}

/// Iterate through a set of Posts and write the files to the output directory
async fn publish_posts(
    settings: &Settings,
    posts: &[Post],
    tera: &Tera,
) -> Result<Option<String>, PublishError> {
    let mut post_iter = posts.iter();
    let current = post_iter.next();
    let prev = post_iter.next();
    let prior = post_iter.next();
    if let Some(current) = current {
        debug!("⦾ publishing new");
        let index = write_post(settings, tera, current, prev, None)
            .await
            .map_err(|e| error!("publish_posts {:?}", e));
        debug!("⦾ publishing prev");
        if let Some(prev) = prev {
            let _ = write_post(settings, tera, prev, prior, Some(current))
                .await
                .map_err(|e| error!("publish_posts {:?}", e));
        }
        debug!("⦾ done");
        return Ok(index.unwrap());
    }
    Ok(None)
}

/// Append the latest Post to the end of the Category file (if it's not already in there)
async fn update_categories(settings: &Settings, posts: &[Post]) -> Result<(), PublishError> {
    if let Some(current) = posts.first() {
        for tag in current.tags.clone() {
            let cat_path = Path::new(&settings.output).join(format!("{}.inc", tag));
            if fs::exists(&cat_path).unwrap_or(false) {
                trace!("🗄️😺 Updating {:?}", &cat_path);
                let content = fs::read_to_string(&cat_path)?;
                if content.contains(&current.link) {
                    debug!(
                        "🗄️ Skipping adding post to {}, already included",
                        current.title
                    );
                    continue;
                }
            }
            // add the link
            info!("🗄️ Generating {:?}...", &cat_path);
            let mut file = fs::File::create(&cat_path)?;
            writeln!(
                &mut file,
                "<li><a href={:?}>{}</a></li>",
                current.link, current.title
            )?;
        }
    }
    Ok(())
}

/// Update the most recent post listing file.
async fn update_archive(settings: &Settings, posts: &Vec<Post>) -> Result<(), PublishError> {
    let archive = Path::new(&settings.output).join("archive.inc");
    let mut file = fs::File::create(&archive)?;
    trace!("🏤 Updating archive: {:?}", &archive);
    writeln!(&mut file, "<ul>")?;
    for post in posts {
        writeln!(
            &mut file,
            "<li><a href={:?}>{}</a></li>",
            post.link, post.title
        )?;
    }
    writeln!(&mut file, "</ul>")?;
    Ok(())
}

/// Update the RSS/CDF file based on the recent posts.
async fn update_rss(
    settings: &Settings,
    posts: &Vec<Post>,
    tera: &Tera,
) -> Result<(), PublishError> {
    let mut context = Context::new();
    let mod_time = if let Some(newest_post) = posts.last() {
        newest_post.date
    } else {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
    context.insert("posts", &posts);
    context.insert("mod_time", &mod_time);
    info!("📰 Updating RSS");
    let file = fs::File::create(Path::new(&settings.output).join("feed"))?;
    tera.render_to("template.rss", &context, file)?;
    info!("📰 Updating CDF");
    let file = fs::File::create(Path::new(&settings.output).join("cdf"))?;
    tera.render_to("template.cdf", &context, file)?;
    Ok(())
}

/// Set the index to point to the most recent file.
async fn set_index(settings: &Settings, latest: &str) -> Result<(), PublishError> {
    info!("📁 Setting index");
    let index = Path::new(&settings.output).join("index.php");
    let _ = fs::remove_file(&index).map_err(|e| {
        warn!("Could not delete old index file {:?}", e);
    });
    std::os::unix::fs::symlink(latest, index)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PublishError> {
    init_logging()?;
    let settings = Settings::new()?;

    let tera = Tera::new(&settings.templates)?;
    let mut posts = get_latest_posts(&settings).await?;
    posts.reverse();
    let posts = get_latest_posts(&settings).await?;
    if settings.new {
        if let Some(latest) = posts.last() {
            info!("Latest num: {}", latest.num);
            let new_post = Post {
                num: latest.num + 1,
                tags: ["crap".to_owned()].to_vec(),
                title: "To Be Determined".to_owned(),
                name: "tbd".to_owned(),
                summary: Some("Remember to change the name of this file to match the short summary!".to_owned()),
                date: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                ..Default::default()
            };
            let new_file = new_post.to_file(Path::new(&settings.source))?;
            if let Ok(editor) = std::env::var("EDITOR") {
                println!("Opening new post: {:?}", &new_file);
                Command::new(editor)
                    .args([new_file])
                    .output()
                    .expect("Could not edit new file");
            }
            return Ok(());
        }
    }
    let index = publish_posts(&settings, &posts, &tera).await?;
    if let Some(index) = index {
        set_index(&settings, &index).await?;
        println!("Published {}", &index);
    }
    debug!("🗄️ updating categories");
    update_categories(&settings, &posts).await?;
    debug!("📰 updating syndication");
    update_rss(&settings, &posts, &tera).await?;
    debug!("🏤 updating archive");
    update_archive(&settings, &posts).await?;
    Ok(())
}
