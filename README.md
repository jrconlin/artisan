# Artisnal Blog

## Introduction

I started using WordPress before it was WordPress. It was a package called `b2`, and it
did a pretty good job for being a light-ish weight blogging platform. Eventually, Wordpress
grew into the most successful blogging platform out there with lots of plug-ins, attack vectors, CVEs and a lunatic CEO.

I also realized a few things (well, aside from my own inertia to get the hell off of WordPress), I really didn't need WordPress.

My blog doesn't have multiple authors. No one has commented on anything in it in over a decade. I constantly fight against dumb features like "links" and the constantly changing edit pane, or really any of the various features WordPress drags in like a cat with Hoarder Syndrome.

So, I decided to move.

The first move was to a fork of WordPress called ClassicPress. It's OK, but again, it's a fork and it's not really that much different than the original.

So I started looking around for other static blog generators, but to do that, I needed to extract my existing data out of the MySQL database. So I created a dumb little script that did just that.

And then i realized, why don't I just build a crappy blog publisher myself?

This is the product of that fever dream.

## The blog

To use this, create a "source" directory that contains files written in MarkDown.

Each file should be named something like "0001_my-first-post.md"

The leading number helps keep things straight, the part after the "_" is the "name" of the post. It's mostly just used in URLs for search, but it's also useful for you when you're scanning the directory.

The content of the file looks like:

```markdown
# Title of the Post
["optional tag", "category", "whatever"]
<!-- Date: 2024-10-20 08:00:00 --> <!-- will use file creation timestamp if missing -->
> Optional summary stuff.
===
## Your Amazing Post Content

_In Glorious Markdown_

```

## Running things.

If you want to do the same thing as me, you might want to start with the `template/index.php` file. Basically, go grab the source for one of your older blog posts, and swap in the
jinja2 elements:

 | *jinja2 item* | what it is |
 | -- | -- |
 | {{url}} | Your blog's URL (handy if you're testing things out) |
 | {{post.link}} | URL to the post |
 | {{post.title}} | The title of your post |
 | {{post.date}} | The Date / time of your post |
 | {{post.categories}}| The various "categories" or "tags" or whatever you want the post to have|
 | {{post.shortlink}} | if you have a "short-link" feature, it's that. |
 | {{post.next}} | URL to the next post |
 | {{post.prev}} | URL to the previous post |

I tried to make as much of this as obvious as I can. Feel free to alter that as much as you like. Why PHP? Because I wanted to use the `<?php include("path_to_file")> ?>` thing for the "latest posts" and a few other things. Don't want that? Killer. Feel free to change things up.

## Rust

The "modern" version is written in rust. There are a few reasons for this, but the biggest one is that my hosting provider doesn't allow me to run python-virtualenv, which really puts a dampener on running anything that's fancy python based. (Stock stuff, sure, but now you're doing weird python tricks to get stuff to work correctly.) Rust builds stand alone executables.

With rust, if you want to build something you can run `cargo`. It tends to be pre-installed on most systems, but if it's not, you can search for downloading `rustup` as a script, and use that to install rust and cargo.

To build:
`cargo build`

This will create a `./target` directory that will contain the executable (either in `./target/debug` or if you use the `--release` flag to build the faster version `./target/release`). If you like, you can create a symbolic link to the executable
(e.g. `ln -s target/debug/publish`) or run `cargo run -- [whatever extra CLI options you want]`

## Python

The first prototype was written in python and is stuffed in the `./python` directory. It includes a `convert.py` that I used to dump my blog's database to a bunch of mark-down style files. Poorly. (I use the fact that markdown accepts HTML, but Wordpress
screwed around with how it stored/wrote/dealt with HTML and styling **A LOT** and I don't really do much conversion. Your older posts will need some hand tending.)

 There are config options, and I even let you write them to a `.toml` file if you want, because there can be a lot of them. This will go read the original database, dump the contents into the markdown files.

It will take a shot at building the PHP files. It doesn't build the archives, though, because I felt lazy, and it only took a few seconds to build 3,000+ posts for me.

## Tags / Categories / whatever

Oh, yeah, those "tag" things. So, Wordpress let you specify "categories" for posts that you could search for. Well, since this is now a super static blog, i figure searching is best accomplished by whatever search engine decided to crawl your blog. Instead, I use those categories to create files that just include a list of the posts that have that tag.

It's very dumb.

I fully expect that these would be wrapped by some HTML cruft to make them look pretty, eventually.

## Writing a new post

Because I am lazy, you can run `publish --new` and the app will auto-generate a new file in your Source directory that is automatically set to the next post number. It will also auto-magically invoke whatever editor you've set using the `EDITOR` environment variable (if it's set) and open that file for you.

Once you've edited the file the way you want, run `publish` to create the new file. (If you want to see what it's up to, you
can use `RUST_LOG=publish=info publish` which will make publish a lot more chatty.)

Remember, if you're not running this on the host machine, you'll need to rsync things up. (e.g.)

```bash
#! /bin/bash -w
rsync -av archive/* YourBox:/path/to/blog/archive
rsync -av imgs/* YourBox:/path/to/blog/imgs
```

