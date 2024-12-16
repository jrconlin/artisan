<!DOCTYPE html>
<html xmlns="https://www.w3.org/1999/xhtml" prefix="og: https://ogp.me/ns#">
  <head profile="https://gmpg.org/xfn/1">
    <title>jr conlin's ink stained banana &raquo; {{ post.title }}</title>
    <!-- Pooh got his head stuck in a honeypot, what a project that must have been, to get his head free from the spider that traps spammers -->
    <meta name="viewport" content="initial-scale=1,width=device-width" />
    <meta property="og:type" content="blog" />
    <meta
      property="og:image"
      content="https://blog.unitedheroes.net/JRS_128x128.jpg"
    />
    <meta name="fediverse:creator" content="@jrconlin@jrconlin.com" />
    <meta name="medium" content="blog" />
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
    <meta name="title" content="{{ post.title }}" />
    <meta name="DC.title" content="{{ post.title }}" />
    <meta property="og:title" content="{{ post.title }}" />
    <meta property="og:description" content="{{ post.summary }}" />
    <meta name="ICBM" content="37.309531,-121.984823" />
    <meta
      name="keywords"
      content="jr conlin,jrconlin,blog,humor,geek,writing,sock monkey,who uses keywords?,tagging is the new keyword"
    />
    <link rel="ICON" type="image/png" href="https://jrconlin.com/favicon.png" />
    <link
      rel="RSS"
      type="application/atom+xml"
      title="Ink-Stained Banana (Atom 1.0)"
      href="https://blog.unitedheroes.net/feed"
    />
    <link rel="author" href="https://unitedheroes.net/authors/jr.php" />
    <link rel="shortlink" href="{{ post.shortlink }}" />
    <link rel="canonical" href="{{ post.link }}" />
    <link
      href="https://fonts.googleapis.com/css?family=Special+Elite&v2"
      rel="stylesheet"
      type="text/css"
    />
    <link
      rel="stylesheet"
      type="text/css"
      href="style.css"
    />
    <?php
      $holiday = "/holiday/".date("md").".css";
      if (file_exists(".".$holiday)) {
      ?>
        <link rel="stylesheet" type="text/css" href="<?= print($holiday) ?>">
      <?php
      }
      ?>
  </head>
  <body>
    <div class="hide" style="border: 2px solid red; padding: 2em">
      <h2>Oops! Something went sideways.</h2>
      <p>
        Looks like the styling got goofed up. Sorry about that, unless it's what
        you wanted. If this isn't what you were looking for, try
        <b>force refreshing</b> your page. You can do that by pressing
        <i>Shift</i> + <i>F5</i>, or holding <i>Shift</i> and clicking on the
        "reload" icon. (It's the weird circle arrow thing "⟳" just above this
        page, usually next to where it says
        <tt>https://blog.unitedheroes.net...)</tt>
      </p>
    </div>

    <div class="topbanner">
      <div
        id="logo"
        onclick="document.location.href='https://blog.unitedheroes.net/'"
      >
        <a href="https://blog.unitedheroes.net/">unitedHeroes.net</a>
      </div>
      <div alt="isn't quite ashamed enough to present" id="label"></div>
      <h1
        id="banner"
        onclick="document.location.href='https://blog.unitedheroes.net/'"
        alt="j r conlin's ink stained banana. This is what happens when you give a monkey a keyboard."
      >
        <a href="https://blog.unitedheroes.net/"
          >jr conlin&#039;s ink stained banana</a
        >
      </h1>
    </div>
    <div class="entryWrap">
      <div class="blogEntry">
        <h2 class="storyHeadline">
          <a href="{{ post.link }}"
            rel="bookmark"
            title="Permanent Link: {{ post.title }}"
            ><span class="storyDate">{{ post.date | date(format="%Y-%m-%d %H:%M:%S") }}</span></a>
          :: {{post.title}}
        </h2>
        <div class="storyCategory">
          <ul class="post-categories">
            {% for tag in post.tags %}
            <li>
              <a href="https://blog.unitedheroes.net/category/{{tag}}" rel="category tag">{{ tag }}</a>
            </li>
            {% endfor %}
          </ul>
      </div>
      <div class="post">
        <div class="storyContent">{{ post.body }}</div>
      </div>
      <div class="shortlink">
        <label>Short Link:</label><span class="link">{{ post.shortlink }}</span><span class="clip">➡️📋</span>
      </div>

      <div class="prevnext">
        <nav class="navigation post-navigation" aria-label="Posts">
          <h2 class="screen-reader-text">Post navigation</h2>
          <div class="nav-links">
            {% if prev %}
            <div class="nav-previous">
              <a href="{{ prev.link }}" rel="prev"><< Previous</a>
            </div>
            {% endif %} {% if next %}
            <div class="nav-next">
              <a href="{{ next.link }}" rel="next">Next >></a>
            </div>
            {% endif %}
          </div>
        </nav>
      </div>
    </div>

    <div class="blogRoll" id="blogRoll">
      <div class="linksBlock">
        <?php include("archive/blogroll.inc"); ?>
      </div>
      <hr />
      <center>
        <div class="rssLinks" id="rssLinks">
          <a
            href="https://blog.unitedheroes.net/cdf"
            class="feed"
            title="Subscribe using the original Syndication Format, CDF"
            ><img
              src="https://blog.unitedheroes.net/feedicon.gif"
              width="16"
              height="16"
              style="border: 0"
            />
            CDF</a
          >
          <a
            href="https://blog.unitedheroes.net//feed"
            class="feed"
            title="Subscribe to this blog with Atom 1.0 and impress people at parties"
            ><img
              src="https://blog.unitedheroes.net/feedicon.gif"
              width="16"
              height="16"
              style="border: 0"
            />
            Atom 1.0</a
          >
        </div>
      </center>
    </div>
    <div class="siteNav" id="siteNav">
      <div class="posts">
        <b>recent posts</b>
        <div class="archives">
          <?php include("archive/archive.inc"); ?>
        </div>
      </div>
      <hr />
      <div id="group">
        <a href="https://unitedheroes.net/group/"><img
            src="https://unitedheroes.net/group/1024/tiny_all.jpg"
            alt="the unitedHeroes Group Photo Project:So far, so odd"
            style="border: 0"
        /></a>
      </div>
      <hr />
      <div class="suggest">
        <div style="font-family: veranda, helveticasize; font-size: -2">
          Lost? Try a <a href="https://maps.google.com">map</a>.<br />
          Confused? Try the <a href="https://blog.unitedheroes.net/faq.php">FAQ</a><br />
          Lonely? Send me a <a href="https://blog.unitedheroes.net/contact_me.php">note</a><br />
        </div>
      </div>
    </div>
    <p class="credit">
    Hosted on <a href="https://click.dreamhost.com/aff_c?offer_id=8&aff_id=16354">Dreamhost</a >.
    <br />
    <span style="font-size: 4px">See our <a href="https://jrconlin.com/b/771">Advertisement Policy</a></span>.
    </p>
    <!-- My utilities -->
    <script
      type="text/javascript"
      src="https://blog.unitedheroes.net/common.js"
    ></script>
    <script type="text/javascript">
      var ch = document.getElementsByTagName("form");
      for (var c1 = 0; c1 < ch.length; c1++) {
        if (ch[c1].challenge) ch[c1].challenge.value = "d0e5081e";
      }
      async function linkToClipboard(text) {
        try {
          await navigator.clipboard.writeText(text);
        } catch (error) {
          console.error(error.message);
        }
      }

      ch = document.getElementsByClassName("shortlink");
      for (var c1 = 0; c1 < ch.length; c1++) {
        ch[c1].addEventListener("click", function (e) {
          let field = e.currentTarget.children[1];
          let clip = e.currentTarget.children[2];
          e.currentTarget.children[1].focus();
          let link = e.currentTarget.children[1].textContent;
          console.info(link);
          linkToClipboard(link);
          clip.style.animation = "2s copied";
        });
      }
    </script>
    <div style="margin: 0 200px 0 100px">
      <script type="text/javascript" src="https://blog.unitedheroes.net/holiday/<?= date("md") ?>.js">
      </script>
    </div>
  </body>
</html>
