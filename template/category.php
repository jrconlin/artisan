<?php
  $category = filter_input(INPUT_GET, "category", FILTER_SANITIZE_STRING, FILTER_FLAG_STRIP_HIGH | FILTER_FLAG_STRIP_LOW | FILTER_FLAG_STRIP_BACKTICK).strtolower();
  $cat_file = "archive/".$category.".inc";
  if !file_exists($cat_file) {
    $category = "crap";
    $cat_file = "archive/crap.inc";
  }
?>
<!DOCTYPE html>
<html xmlns="https://www.w3.org/1999/xhtml" prefix="og: https://ogp.me/ns#">
  <head profile="https://gmpg.org/xfn/1">
    <title>jr conlin&#039;s ink stained banana &raquo; <? print($category) ?></title>
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
    <div class="topbanner">
      <div
        id="logo"
        onclick="document.location.href='/'"
      >
        <a href="/">unitedHeroes.net</a>
      </div>
      <div alt="isn't quite ashamed enough to present" id="label"></div>
      <h1
        id="banner"
        onclick="document.location.href='/'"
        alt="j r conlin's ink stained banana. This is what happens when you give a monkey a keyboard."
      >
        <a href="/"
          >jr conlin&#039;s ink stained banana</a
        >
      </h1>
    </div>
    <div class="entryWrap">
      <div class="blogEntry">
      <h1><?php print($category); ?>
    <ul class="categories">
      <? include($cat_file); ?>
    </ul>
    </div>

    <div class="blogRoll" id="blogRoll">
      <div class="linksBlock">
        <b>Blogs of note</b>
        <hr />
        <b>personal</b>
        <a href="http://christopherconlin.com/" rel="brother"
          >Christopher Conlin USMC</a
        >
        <a
          href="http://www.henriettesherbal.com/blog/index.php"
          rel="friend I-think-she's-spiffy"
          >Henriette's Herbal Blog</a
        >
        <a rel="me" href="https://soc.jrconlin.com/@jrconlin"
          >My Mastodon musings</a
        >
        Where have all the good blogs gone?
        <hr />
        <b>geek</b>
        <a
          href="http://ultramookie.com/"
          rel="colleague met-me-and-didn't-run-away-screaming"
          >ultramookie</a
        >
        <br />
      </div>
      <hr />
      <center>
        <div class="rssLinks" id="rssLinks">
          <a
            href="{{root}}/cdf"
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
            href="{{url}}/atom.xml"
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
          <?php include("archive.inc"); ?>
        </div>
      </div>
      <hr />
      <div id="group">
        <a href="https://unitedheroes.net/group/"
          ><img
            src="https://unitedheroes.net/group/1024/tiny_all.jpg"
            alt="the unitedHeroes Group Photo Project:So far, so odd"
            style="border: 0"
        /></a>
      </div>
      <hr />
      <div class="suggest">
        <div style="font-family: veranda, helveticasize; font-size: -2">
          Lost? Try a <a href="https://maps.google.com">map</a>.<br />
          Confused? Try the
          <a href="https://blog.unitedheroes.net/faq.php">FAQ</a><br />
          Lonely? Send me a
          <a href="https://blog.unitedheroes.net/contact_me.php">note</a
          ><br />
        </div>
      </div>
    </div>
    <p class="credit">
    Hosted on
    <a href="https://click.dreamhost.com/aff_c?offer_id=8&aff_id=16354"
      >Dreamhost</a
    >.
    <br />
    <span style="font-size: 4px"
      >See our
      <a href="https://jrconlin.com/b/771">Advertisement Policy</a></span
    >.
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
