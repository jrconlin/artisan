#![forbid(unsafe_code)]
use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
    time::SystemTime,
};

use chrono::prelude::*;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
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

/// Settings what set the things to do what you want.
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Optional config file. So you don't have to specify them all. (-c, --config | config.toml)
    pub config: Option<String>,
    /// Directory containing the jinja templates (-t, --templates | "./templates")
    pub templates: String,
    /// Where to write the created files (-o , --output | "./archive")
    pub output: String,
    /// Where the markdown files are (-s, --source | "./source")
    pub source: String,
    /// How many files should be considered "recent"? (-r, --recent | 10)
    pub recent: u32,
    /// The name of the blog ( --blog_name )
    pub blog_name: String,
    /// The URL for the blog (--url)
    pub url: String,
    /// Do you have a short URL? If not, reuse the `url` (--short_url)
    pub short_url: Option<String>,
    /// Order by creation time or post number? (--by_time | False)
    pub by_time: bool,
    /// Order by file name (--by_name | True)
    pub by_name: bool,
}

impl Settings {
    // All because every arg parser either can't do bools or makes it
    // weirdly complex to do them. Plus, I want the option of reading
    // from a config file.
    pub fn new() -> Result<Self, PublishError> {
        let mut args: Vec<String> = env::args().collect();
        let mut settings = Settings::default();

        while !args.is_empty() {
            if let Some(op) = args.pop() {
                match op.as_str() {
                    "-c" | "--config" => {
                        let file = args.pop();
                        if file.is_none() {
                            return Err(PublishError::SettingsError(
                                "No config file specified".to_owned(),
                            ));
                        }
                        let filename = file.unwrap();
                        let file = Path::new(&filename);
                        if !file.exists() {
                            return Err(PublishError::SettingsError(format!(
                                "Config file not found {:?}",
                                filename
                            )));
                        }
                        let handle = fs::read_to_string(file).map_err(|e| {
                            PublishError::SettingsError(format!(
                                "Could not read config file: {:?} {:?}",
                                filename, e
                            ))
                        })?;
                        settings = toml::from_str(&handle).map_err(|e| {
                            PublishError::SettingsError(format!(
                                "Could not parse config file contents: {:?}",
                                e
                            ))
                        })?;
                    }
                    "-t" | "--templates" => {
                        if let Some(val) = args.pop() {
                            settings.templates = val;
                        }
                    }
                    "-o" | "--output" => {
                        if let Some(val) = args.pop() {
                            settings.output = val;
                        }
                    }
                    "-s" | "--source" => {
                        if let Some(val) = args.pop() {
                            settings.source = val;
                        }
                    }
                    "-r" | "--recent" => {
                        if let Some(val) = args.pop() {
                            settings.recent = val.parse::<u32>().map_err(|_| {
                                PublishError::SettingsError(
                                    "Could not parse recent count".to_owned(),
                                )
                            })?;
                        }
                    }
                    "--blog_name" => {
                        if let Some(val) = args.pop() {
                            settings.blog_name = val;
                        }
                    }
                    "--url" => {
                        if let Some(val) = args.pop() {
                            settings.url = val;
                        }
                    }
                    "--short-url" => {
                        if let Some(val) = args.pop() {
                            settings.short_url = Some(val);
                        }
                    }
                    "--by_time" => {
                        settings.by_time = true;
                        settings.by_name = false;
                    }
                    "--by_name" => {
                        settings.by_time = false;
                        settings.by_name = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(settings)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            config: Some("config.toml".to_owned()),
            templates: "template/*".to_owned(),
            output: "archive".to_owned(),
            source: "source".to_owned(),
            recent: 10,
            blog_name: "jr conlin's ink stained banana".to_owned(),
            url: "https://blog.unitedheroes.net".to_owned(),
            short_url: Some("https://jrconl.in/b".to_owned()),
            by_time: false,
            by_name: true,
        }
    }
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
    /// The content of the post
    pub body: String,
    /// An optional summary (mostly used by RSS.)
    pub summary: Option<String>,
    /// The URL/file safe name for this post
    pub name: String,
    /// The date for the post (default to the atime of the file.)
    pub date: String,
}

impl Post {
    /// Construct the post from the file.
    async fn from_file(settings: &Settings, filepath: String) -> Result<Self, PublishError> {
        let mut result = Self::default();
        let handle = fs::File::open(&filepath).unwrap();
        let atime = handle.metadata().unwrap().created().unwrap();
        let file: io::Lines<io::BufReader<fs::File>> = io::BufReader::new(handle).lines();
        info!("✍ {:?}", &filepath);
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
                        result.tags_from_str(&line)?;
                    }
                    if line.starts_with("<!-- Date:") {
                        result.date_from_str(&line)?;
                    }
                    if line.starts_with("# ") {
                        result.title_from_str(&line)?;
                    }
                    if line.starts_with("\"") {
                        result.summary = Some(line.trim().to_owned());
                    }
                } else {
                    body.push(line);
                }
            } else {
                break;
            }
        }
        if result.date.is_empty() {
            result.date = time_str(&atime);
        }
        result.body = body.join("");
        Ok(result)
    }

    /// Read the tags from a string (this is because serde_json doesn't handle
    /// strings that are a list. This may be due to a long standing security issue
    /// with JSON not dealing with sets well.)
    fn tags_from_str(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        trace!("tags: {:?}", &line);
        // Sadly, serde chokes on just a set, so I can't use that.
        // Have to break these apart the manyal way.
        self.tags = line
            .to_owned()
            .replace(['[', ']', '"'], "")
            .split(",")
            .map(|v| v.trim().to_owned())
            .collect();
        Ok(self)
    }

    /// Extract a date from a POST header string
    fn date_from_str(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        trace!("date: {:?}", &line);
        let re = Regex::new(r"<!-- (Date:)? (?<ts>.*) -->").expect("Date Regex altered");
        self.date = re.replace(line, "$ts").into_owned();
        Ok(self)
    }

    /// Extract the post title from the header string. (Basically just strip off the leading "# ")
    fn title_from_str(&mut self, line: &str) -> Result<&mut Self, PublishError> {
        self.title = line.strip_prefix("# ").unwrap().to_owned();
        Ok(self)
    }
}

/// Serialize the Post into something we can write to a file and (hopefully, read back later.)
impl From<Post> for String {
    fn from(post: Post) -> String {
        let tags = serde_json::json!(post.tags).to_string();
        let date = if post.date.is_empty() {
            time_str(&SystemTime::now())
        } else {
            post.date
        };
        format!(
            "# {}\n{}\n{}\n{}\n===\n{}",
            post.title,
            date,
            tags,
            post.summary.unwrap_or_default(),
            post.body,
        )
    }
}

/// Return a normalized Time String for things.
fn time_str(time: &SystemTime) -> String {
    <DateTime<Local>>::from(*time)
        .format(TIME_FORMAT)
        .to_string()
}

/// Get a list of files based on the leading number. (ideally, this should optionally sort based
/// on the atime of the files like the python version does, but that can wait.
fn get_latest_files(settings: &Settings) -> Result<Vec<String>, PublishError> {
    let mut files: Vec<String> = Vec::new();
    let re = Regex::new(r"^[0-9]{4}.*\.md$").expect("WTF: Regex failed to compile");
    trace!("Reading source... {}", &settings.source);
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
fn write_file(
    settings: &Settings,
    tera: &Tera,
    current: &Post,
    prev: Option<&Post>,
    next: Option<&Post>,
) -> Result<Option<String>, PublishError> {
    let path = Path::new(&settings.output).join(format!("{}.php", &current.num));
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
    trace!("Writing ...{} to {:?}", &current.title, &path);
    // Current post file
    let name = path.to_str().map(|v| v.to_owned());
    let current_file = fs::File::create(path)?;
    tera.render_to("index.php", &context, current_file)
        .map_err(PublishError::TeraError)?;
    Ok(name)
}

/// Iterate through a set of Posts and write the files to the output directory
fn publish_posts(
    settings: &Settings,
    posts: &[Post],
    tera: &Tera,
) -> Result<Option<String>, PublishError> {
    let mut post_iter = posts.iter();
    let current = post_iter.next();
    let prev = post_iter.next();
    let prior = post_iter.next();
    if let Some(current) = current {
        let index = write_file(settings, tera, current, prev, None)?;
        if let Some(prev) = prev {
            write_file(settings, tera, prev, prior, Some(current))?;
        }
        return Ok(index);
    }
    Ok(None)
}

/// Append the latest Post to the end of the Category file (if it's not already in there)
fn update_categories(settings: &Settings, posts: &[Post]) -> Result<(), PublishError> {
    if let Some(current) = posts.first() {
        for tag in current.tags.clone() {
            let cat_path = Path::new(&settings.output).join(format!("{}.inc", tag));
            if fs::exists(&cat_path).unwrap_or(false) {
                let content = fs::read_to_string(&cat_path)?;
                if content.contains(&current.link) {
                    info!(
                        "Skipping adding post to {}, already included",
                        current.title
                    );
                    continue;
                }
            }
            // add the link
            info!("Generating {:?}...", &cat_path);
            let mut file = fs::File::options()
                .append(true)
                .create(true)
                .open(&cat_path)?;
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
fn update_archive(settings: &Settings, posts: &Vec<Post>) -> Result<(), PublishError> {
    let mut file = fs::File::create(Path::new(&settings.output).join("archive.inc"))?;
    for post in posts {
        writeln!(
            &mut file,
            "<li><a href={:?}>{}</a></li>",
            post.link, post.title
        )?;
    }
    Ok(())
}

/// Update the RSS/CDF file based on the recent posts.
fn update_rss(settings: &Settings, posts: &Vec<Post>, tera: &Tera) -> Result<(), PublishError> {
    let mut context = Context::new();
    let mod_time = if let Some(newest_post) = posts.last() {
        newest_post.date.clone()
    } else {
        time_str(&SystemTime::now())
    };
    let mut blog = HashMap::<&str, &str>::new();
    let rss_link = format!("{}/feed", &settings.url);
    let cdf_link = format!("{}/cdf", &settings.url);
    blog.insert("url", &settings.url);
    blog.insert("rss_link", &rss_link);
    blog.insert("cdf_link", &cdf_link);
    blog.insert("title", &settings.blog_name);
    context.insert("posts", &posts);
    context.insert("mod_time", &mod_time);
    context.insert("blog", &blog);
    let file = fs::File::create(Path::new(&settings.output).join("feed"))?;
    tera.render_to("template.rss", &context, file)?;
    let file = fs::File::create(Path::new(&settings.output).join("cdf"))?;
    tera.render_to("template.cdf", &context, file)?;
    Ok(())
}

/// Set the index to point to the most recent file.
fn set_index(settings: &Settings, index: &str) -> Result<(), PublishError> {
    std::os::unix::fs::symlink(index, Path::new(&settings.output).join("index.php"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PublishError> {
    let settings = Settings::new().unwrap();

    let tera = Tera::new(&settings.templates)?;
    let mut posts = get_latest_posts(&settings).await?;
    posts.reverse();
    let index = publish_posts(&settings, &posts, &tera)?;
    info!("updating categories");
    update_categories(&settings, &posts)?;
    info!("updating rss");
    update_rss(&settings, &posts, &tera)?;
    info!("updating archive");
    update_archive(&settings, &posts)?;
    info!("setting index");
    if let Some(index) = index {
        set_index(&settings, &index)?;
    }
    Ok(())
}
