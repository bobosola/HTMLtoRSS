# HTML to RSS

This is a simple text grabber to create an RSS feed `<item>` element from a local file system HTML page, such as a blog page. It does the following:

* grabs all the text inside the `<main>` element ignoring the first `LINES_TO_CUT` number of lines to allow for the removal of headings or anything else you don't need
* converts all elements with relative paths (i.e. those with `href`, `src`, or `srcset` attributes) to full URL paths so that they will work in an external feed reader
* removes all extraneous white space used for formatting
* copies the result to clipboard as a populated RSS `<item>` element (as described below) ready for pasting directly into an `rss.xml` file

The following two constants must be set in `src/main.rs`:
* `BASE_URL` - the URL for converting relative paths to full URLs
* `LINES_TO_CUT` - the number of lines to ignore from the start of the `<main>` element

## Application description

The application requires (i) an html file path and (ii) the title text as arguments. It produces a populated `<item>` element like this :

```xml
<item>
    <title>First day at School</title>
    <link>https://yoursite.com/blog/first_day_at_school.htm</link>
    <description>
        <![CDATA[
            <img src="https://yoursite/com/blog/images/school.jpg" alt="1st day as school" width="500" height="600">Today we took our son to school for the very first time .... etc.
        ]]>
    </description>
    <pubDate>Sat, 11 Oct 2025 16:44:06 GMT</pubDate>
    <guid>cdcda14c-63e8-4b29-8fae-67835ad92dd3</guid>
</item>
```

It may seem odd to put the entire page content into the `<description>` element, but it's common practice these days, presumably because of the popularity of automated blog authoring tools. And it's allowed in the [RSS 2.0 Specifications](https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt). So I have adopted this practice here. It also has the advantage of allowing people to read successive articles in their entirety in a feed reader without having to jump in and out of a browser.

## Build steps

* [Install Rust](https://rust-lang.org/tools/install/)
* `git clone https://github.com/bobosola/HTMLtoRSS.git`
* `cd HTMLtoRSS`
* `cargo build --release`

The executable will be in `target/release` as `HTMLtoRSS`. To use the app from anywhere, I suggest moving it to `/usr/local/bin`:

* `sudo mv target/release/HTMLtoRSS /usr/local/bin`

## Usage

`HTMLtoRSS html-file-path title` where:
* `html-file-path` is the file name of the html file you want to grab the content from relative to the current working directory
* `title` is your RSS feed article title

For example, assuming you are in your site root directory and the file to grab the content from is in the `blog` subdirectory:

`HTMLtoRSS blog/my_latest_article.htm "My article title"`

The application will confirm success by printing:

`✅ Copied to clipboard!`

## Producing a simple RSS feed

This is a quick how-to for anyone new to creating an RSS feed. You will need the following:

1) An `rss.xml` file somewhere on your site:

```XML
<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <atom:link href="https://yoursite.com/blog/rss.xml" rel="self" type="application/rss+xml" />
        <title>My Mighty Blog</title>
        <link>https://yoursite.com/blog</link>
        <description>A blog about my dull life</description>
        <language>en-uk</language>
        <item>...</item>
        <item>...</item>
        ...
    </channel>
</rss>
```

2) A `<link>` in the `<head>` of your home page linking to `rss.xml`:
```html
<link rel="alternate" type="application/rss+xml" title="RSS Feed for my Blog" href="/blog/rss.xml">
```

3) An RSS logo of your choice in a suitable location also linking to `rss.xml` file:
```html
<a href="/blog/rss.xml"><img src="/blog/images/rss_logo.gif" alt="RSS logo" width="36" height="14"></a>
```

Items 2) and 3) allow interested people to subscribe to your feed either by:
* pasting your home page URL into their feed reader or
* by clicking the RSS logo and copying the URL to their feed reader

You can then use this application to produce populated `<item>` elements from your site HTML pages and paste each one into the `<channel>` element.
