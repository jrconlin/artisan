#! python3

"""
Convert a WordPress blog to a bunch of MarkDown files and PHP files.

"""
import argparse
import jinja2
import asyncio
import os
import logging
import toml
import typing
import re

from MySQLdb import _mysql as mysql
from urllib.parse import urlparse


def connect(dsn):
    parsed = urlparse(dsn)
    if parsed.scheme != "mysql":
        raise Exception("Invalid DSN")
    config = dict()
    if "@" in parsed.netloc:
        (up, config["host"]) = parsed.netloc.split("@")
        if ":" in up:
            (config["user"], config["password"]) = up.split(":")
        else:
            config["user"] = up
        if parsed.path is not None:
            config["database"] = parsed.path.strip("/")
    return mysql.connect(**config)


def fetch_posts(config):
    db = connect(config.dsn)
    query = f"""SELECT
        ID AS "post_id",
        post_date,
        post_title,
        post_content,
        post_name
        FROM {config.prefix}posts
        WHERE post_status="publish"
            AND post_type="post"
        ORDER BY post_id; """

    db.query(query)
    return db.use_result()


def fetch_tags(config, id):
    db = connect(config.dsn)
    query = f"""
    SELECT
        {config.prefix}posts.ID,
        {config.prefix}terms.name
    FROM {config.prefix}terms
    JOIN {config.prefix}term_relationships ON {config.prefix}terms.term_id = {config.prefix}term_relationships.term_taxonomy_id
    JOIN {config.prefix}posts ON {config.prefix}posts.ID = {config.prefix}term_relationships.object_id
    WHERE {config.prefix}posts.ID = {id};
    """
    db.query(query)
    result = db.use_result()
    return [row[1].decode() for row in result.fetch_row()]


async def dump_md(
    log: logging.Logger, config: argparse.Namespace, current: dict[str, typing.Any]
):
    log.debug(
        f"""📃✍ Creating "{config.output_dir}/{current["id"]:04}_{current["name"]}.md" """
    )
    with open(
        f"""{config.md_dir}/{current["id"]:04}_{current["name"]}.md""",
        "w",
        encoding="utf-8",
    ) as out:
        out.write(f"""# {current["title"]}\n""")
        out.write(f"""{current["tags"]}\n""")
        out.write(f"""<!-- Date: {current["date"]} -->\n""")
        out.write(f"""===\n""")
        out.write(f"""{current["body"]}""")


async def dump_html(
    log: logging.Logger,
    config: argparse.Namespace,
    templ: jinja2.Template,
    current,
    prev,
    next,
):
    log.debug(
        f"""🕸✍ Creating "{config.output_dir}/{current["id"]:04}_{current["name"]}.php" """
    )
    with open(
        f"{config.output_dir}/{current["id"]:04}.php", "w", encoding="utf-8"
    ) as out:
        log.info(f"Writing... {current["title"]}")
        out.write(
            templ.render(
                {
                    "post": current,
                    "next": next,
                    "prev": prev,
                    "url": config.url,
                    "short_url": config.short_url,
                }
            )
        )


async def dump_tags(
    log: logging.Logger, config: argparse.Namespace, current: dict[str:any]
):
    for tag in current["tags"]:
        tag_file = tag.lower().replace(" ", "_").strip("'\"")
        with open(f"{config.output_dir}/{tag_file}.inc", "a") as file:
            file.write(
                f"""<li><a href="{current["link"]}">{current["title"]}</a></li>\n"""
            )


def make_shortlink(url: str, id: int) -> str:
    return f"""{url}/{id:04}"""


async def amain(log: logging.Logger, config: argparse.Namespace):
    jenv = jinja2.Environment(loader=jinja2.FileSystemLoader(config.template_dir))
    page_templ = jenv.get_template(f"index.php")
    cursor = fetch_posts(config)
    prev = None
    current = None
    next = None
    # TODO: strip the host out of the config.url for this crap.
    de_wp = re.compile(r"https?://blog\.unitedheroes\.net/wp-content/uploads/\d+/\d+")
    while True:
        try:
            ((post),) = cursor.fetch_row()
            tags = fetch_tags(config, post[0].decode())
            body = post[3].decode()
            if de_wp.search(body):
                log.debug(f"De-wp {post[0]}")
                body = de_wp.sub("/imgs", body)
            next: dict[str, typing.Any] = dict(
                id=int(post[0].decode()),
                date=post[1].decode(),
                title=post[2].decode(),
                body=body,
                name=post[4].decode(),
                tags=tags,
            )
            next["shortlink"] = make_shortlink(config.short_url, next["id"])
            next["link"] = f"""{config.url}/{next['id']:04}_{next['name']}"""
        except ValueError as ex:
            log.info("Last?")
            next = None
        if next is None and current is None:
            break
        if current is None:
            current = next
            continue
        if prev:
            current["prev"] = prev["link"]
        if next:
            current["next"] = next["link"]
        if current is not None:
            await dump_md(log=log, config=config, current=current)
            await dump_tags(log=log, config=config, current=current)
            await dump_html(log, config, page_templ, current, next, prev)

        prev = current
        current = next
        next = None


def init_logging() -> logging.Logger:
    log = logging.basicConfig(
        level=logging.getLevelNamesMapping().get(
            os.environ.get("PYTHON_LOG", "info").upper(), "INFO"
        )
    )
    return logging.getLogger("static")


def config(log: logging.Logger) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Static Site Builder")
    parser.add_argument("-c", "--config", help="Configuration file")
    parser.add_argument(
        "--template_dir", default="template", help="Directory containing templates"
    )
    parser.add_argument(
        "--output_dir", default="output", help="Directory to write files"
    )
    parser.add_argument("--md_dir", default="source", help="Source Markdown files")
    parser.add_argument("--dsn", default="mysql://uh:uh@localhost/unitedheroes")
    parser.add_argument("--prefix", default="jr_wp_", help="Wordpress table prefix")
    parser.add_argument(
        "--archive_count", default=10, help="Number of posts for the 'archive'"
    )
    parser.add_argument(
        "--url",
        default="https://blog.unitedheroes.net",
        help="The location of your blog",
    )
    parser.add_argument(
        "--short_url",
        default="https://jrconl.in/b",
        help="The optional short URL for your blog",
    )
    args = parser.parse_args()
    if args.config is not None:
        log.debug(f"Reading from {args.conf}")
        with open(args.config, "r") as conf:
            parser.set_defaults(**toml.load(conf))
            args = parser.parse_args()
    if args.short_url is None:
        args.short_url = args.url
    return args


def main(log, config):
    asyncio.run(amain(log=log, config=config))


if __name__ == "__main__":
    log = init_logging()
    main(log=log, config=config(log))
