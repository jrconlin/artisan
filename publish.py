#! python3

#
# Scan and publish the latest blog posts.
#
# This does a lot of dumb things. That's mostly by design.
# It looks for the last 10 or so files and builds stuff off of those.
# This uses MarkDown for post files, which have a format
#
# ####_the_name_for_the_url.md
#
# the content of the file has a "prefix" format of
#
# """Markdown
#
# # Post Title
# [tag, tag, tag]
# <!-- Date YYYY-mm-dd HH:MM:SS -->
# ===
#
# _Your blog content here_.
#
# """
#

import argparse
import os
import glob
import logging
import markdown
import jinja2
import toml
from datetime import datetime


class Post:
    link: str
    title: str
    num: int
    tags: list[str]
    body: str
    name: str
    date: datetime

    def __init__(self, log: logging.Logger, config: argparse.Namespace, file: str):
        # get post number and name from file name.
        # extract meta data from file header:
        # # Post Title
        # <!-- Date: {Post Date} -->
        # ["post tag", ...]
        # ===
        # Post body...
        #
        self.title = ""
        self.tags = []
        self.body = ""
        self.date = datetime.fromtimestamp(os.path.getctime(file))
        self.num = 0
        (num, name) = os.path.split(file)[1].split("_", 1)
        self.num = int(num)
        self.name = os.path.splitext(name)[0]
        self.link = f"""{config.url}/{self.num:04}_{self.name}"""
        self.shortlink = f"""{config.short_url}/{self.num:04}"""
        log.debug(f"Post {self.num:04} => {self.name}")
        content = open(file, "r").readlines()
        while True:
            line = content.pop(0).strip()
            if not len(line):
                continue
            if line.startswith("# "):
                self.title = line[2:]
                continue
            if line.startswith("["):
                self.tags = [tag.strip() for tag in line.strip("[']").split(",")]
                continue
            if line.startswith("<!-- Date:"):
                self.date = datetime.fromisoformat(line[10:-3].strip())
                continue
            if line.startswith("==="):
                break
        self.body = markdown.markdown(
            "".join(content),
            extensions=["codehilite", "fenced_code"],
        )


def get_latest_posts(log: logging.Logger, config: argparse.Namespace) -> list[Post]:
    # Return a list of the 10? most recent files from the source directory.
    posts = []
    files = list(filter(os.path.isfile, glob.glob(f"{config.source}/*.md")))
    log.debug(f"Found {len(files)} files")
    if config.by_time:
        files.sort(key=os.path.getctime)
    if config.by_name:
        files.sort()
    for file in files[-config.recent_count :]:
        posts.append(Post(log, config, file))
    log.debug(f"returning {config.recent_count} posts")
    return posts[-config.recent_count :]
    pass


def get_config(log: logging.Logger) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Publish a post and update the various files"
    )
    parser.add_argument("-c", "--config", help="Configuration file")
    parser.add_argument(
        "--template_dir", default="template", help="Directory containing templates"
    )
    parser.add_argument(
        "--output_dir", default="output", help="Directory to write files"
    )
    parser.add_argument(
        "--md_dir", default="source", dest="source", help="Source Markdown files"
    )
    parser.add_argument(
        "--recent_count", default=10, help='Number of "most recent" posts'
    )
    parser.add_argument(
        "--blog_name",
        default="jrconlin's ink stained banana",
        help="The title of your blog",
    )
    parser.add_argument(
        "--url", default="https://blog.jrconlin.com", help="The location of your blog"
    )
    parser.add_argument(
        "--short_url",
        default="https://jrconl.in/b",
        help="The optional short URL for your blog",
    )
    parser.add_argument(
        "--by_time",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Sort posts by creation time",
    )
    parser.add_argument(
        "--by_name",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Sort posts by name",
    )
    args = parser.parse_args()
    if args.config is not None:
        log.debug(f"Reading from {args.conf}")
        with open(args.config, "r") as conf:
            parser.set_defaults(**toml.load(conf))
            args = parser.parse_args()
    args.jinja = jinja2.Environment(loader=jinja2.FileSystemLoader(args.template_dir))

    return args


def get_logging() -> logging.Logger:
    log = logging.basicConfig(
        level=logging.getLevelNamesMapping().get(
            os.environ.get("PYTHON_LOG", "debug").upper(), "INFO"
        )
    )
    return logging.getLogger("static")


def update_categories(log: logging.Logger, config: argparse.Namespace, post: Post):
    """Add post to the end of the list of posts for the mentioned categories"""
    if len(post.tags) > 0:
        log.info("🐱 Updating category files")
    for tag in post.tags:
        tag_file = tag.lower().replace(" ", "_").strip("'\"")
        cat_file = f"{config.output_dir}/{tag_file}.html"
        # don't include the post in the cat file if it's already there.
        with open(cat_file, "r") as file:
            if post.link in file.read():
                log.debug(f"😿 Skipping duplicate cat {tag_file}")
                continue
        with open(f"{config.output_dir}/{tag_file}.html", "a") as file:
            log.debug(f"😸 Adding to cat {tag_file}")
            file.write(f"""<li><a href="{post.link}">{post.title}</a></li>\n""")


def update_rss(log: logging.Logger, config: argparse.Namespace, posts: list[Post]):
    # use jinja2 to compose the RSS file from posts
    mod_time = datetime.now().isoformat()
    blog = dict(
        name=config.blog_name or "jr conlin's ink stained banana",
        url=config.url or "https://blog.jrconlin.com/",
    )

    log.info(f"🗞 Writing RSS...")
    templ = config.jinja.get_template("template.rss")
    with open(f"{config.output_dir}/feed", "w", encoding="utf-8") as rss:
        templ.render(
            {
                "mod_time": mod_time,
                "blog": blog,
                "posts": posts,
            }
        )
    # use jinja2 to compose the CDF file from posts
    log.info(f"🗞 Writing CDF...")
    templ = config.jinja.get_template("template.cdf")
    with open(f"{config.output_dir}/cdf", "w", encoding="utf-8") as rss:
        templ.render(
            {
                "mod_time": mod_time,
                "blog": blog,
                "posts": posts,
            }
        )

    pass


def update_archive(log: logging.Logger, config: argparse.Namespace, posts: list[Post]):
    # list the most recent posts in reverse chronological order
    log.info(f"📚 Updating recent post list")
    with open(f"{config.output_dir}/archive.inc", "w") as file:
        file.write("""<ul class="posts">""")
        for post in posts[::-1]:
            file.write(f"""<li><a href="{post.link}">{post.title}</a></li>\n""")
        file.write("</ul>")


def publish_post(log: logging.Logger, config: argparse.Namespace, posts: list[Post]):
    # use jinja2 to compose the PHP files.
    # We need the prior two posts so that we can update those to point to the newest.
    templ = config.jinja.get_template("index.php")
    post = posts[-1]
    prev = posts[-2]
    prior = posts[-3]
    index = f"{config.output_dir}/{post.num:04}.php"
    log.debug(f"""🕸✍ Creating "{index}" """)
    with open(index, "w", encoding="utf-8") as out:
        log.info(f"Writing... {post.title}")
        out.write(
            templ.render(
                {
                    "post": post,
                    "prev": prev,
                    "url": config.url,
                    "short_url": config.short_url,
                }
            )
        )
    log.debug(f"""🕸✍ Updating "{config.output_dir}/{prev.num:04}""")
    with open(f"{config.output_dir}/{prev.num:04}.php", "w", encoding="utf-8") as out:
        log.info(f"Updating... {prev.title}")
        out.write(
            templ.render(
                {
                    "post": prev,
                    "prev": prior,
                    "next": post,
                    "url": config.url,
                    "short_url": config.short_url,
                }
            )
        )
    return index


def set_index(log: logging.Logger, config: argparse.Namespace, index: str):
    link = f"{config.output_dir}/index.php"
    if os.path.exists(link):
        os.unlink(link)
    os.link(index, link)


def main():

    log = get_logging()
    # get the latest (not `draft_*.md`) file
    config = get_config(log)
    (posts) = get_latest_posts(log, config)
    index = publish_post(log, config, posts[-3:])
    update_categories(log, config, posts[-1])
    update_rss(log, config, posts)
    update_archive(log, config, posts)
    set_index(log, config, index)


if __name__ == "__main__":
    main()
