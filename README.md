# HTMLtoRSS

Rough and ready text grabber largely coded by `qwen3-30b-a3b-instruct-2507-mlx` to create RSS feed entries from a local file system HTML page.

Producing a simple RSS feed is easy. You will need:

1) An `rss.xml` file somewhere like this:

```XML
<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <atom:link href="https://{your blog url}/rss.xml" rel="self" type="application/rss+xml" />
        <title>My Mighty Blog</title>
        <link>https://{your blog url}</link>
        <description>A blog about my dull life</description>
        <language>en-uk</language>
        {item elements go here}
    </channel>
</rss>
```

2) Something like this in the `<head>` element of your home page:
```html
<link rel="alternate" type="application/rss+xml" title="RSS Feed for my Blog" href="/blog/rss.xml">
```

3) An RSS logo of your choice linking to the `rss.xml` file on your blog index page:
```html
<a href="/blog/rss.xml"><img src="/blog/images/pic_rss.gif" alt="RSS logo" width="36" height="14"></a>
```

This allows interested readers to subscribe to your feed either by pasting your home page URL into their feed reader or by clicking the RSS logo on your blog index page.

The app produces an `<item>` element given the name of an html file and a title. It produces something like this copied to clipboard ready for pasting into the `rss.xml` file inside the `<channel>` element:

```xml
<item>
    <title>Your article title</title>
    <link>{url to article web page}</link>
    <description>
        <![CDATA[
            {curated text from the html page}
        ]]>
    </description>
    <pubDate>{the current date and time in the approved RSS format}</pubDate>
    <guid>{unique identifer}</guid>
</item>
```

This was written to convert blog pages on my own site to enable the page content to be easily pasted into an `rss.xml` file. It does the following:

* grabs all the text inside the `<main>` element
* ignores the first three lines (unwanted `H1` and `H2` tags in my case)
* converts all relative paths with `href`, `src`, or `srcset` attributes to full URL paths so that they will work in an external feed reader
* removes all extraneous white space used for formatting
* creates an RSS `<item>` element as described above
* copies the result to clipboard ready for pasting directly into the `rss.xml` file

The following are set as constants in `src/main.rs`:
* the base URL for the full path
* the number of lines to cut

## To Build

* install Rust
* `git clone https://github.com/bobosola/HTMLtoRSS.git`
* `cd HTMLtoRSS`
* `cargo build --release`

The executable will be in `/target/release` as `HTMLtoRSS`.

## Usage

`HTMLtoRSS html-file-path title` where:
* `html-file-path` is the file name of the html file you want to grab the content from relative to the current working directory
* `title` is your RSS feed article title

For example: `HTMLtoRSS blog/my_latest_article.htm "My article title"`

I suggest making an alias to the app in `/usr/local/bin` so that you can call it from anywhere:
`sudo ln -s /your/full/path/to/HTMLtoRSS/target/release/HTMLtoRSS /usr/local/bin/HTMLtoRSS`
