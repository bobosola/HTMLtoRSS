# HTML to RSS

This is a command line application to create an RSS feed item from a section of:
* a local static HTML page, or
* a URL pointing to a well-formed HTML5 webpage

There is also a demo RSS.xml file with instructions below for setting up an RSS feed from scratch.

The application inserts the extracted text into the RSS.xml file as a new `item` element. It is designed for occasional bloggers who create static HTML pages and who wish to add an RSS feed to their site.

## Functionality

`HTMLtoRSS` does the following:
* grabs all the text inside a CSS selector&mdash;the default is "main" to retrieve content from the the `<main>` element, but this can be overridden
* uses the first `<h1>` text as the item title text, but this can be overridden
* requires a base URL to be supplied to convert all relative `href`, `src`, and `srcset` attributes to absolute URLs so that they will work in an external feed reader
* removes all extraneous whitespace used for formatting
* optionally ignores a number of lines to allow for the removal of unwanted headings etc.
* copies the result into the RSS.xml file as a new `item` with the date and time set to the time of insertion with a new GUID.

## Output

The application produces a populated `<item>` element something like this :

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

It may seem unintuitive to put the extracted page content into the `<description>` element, but it's common practice, and is allowed in the [RSS 2.0 Specifications](https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt). This approach has the advantage of allowing people to read successive articles in their entirety in a feed reader without having to jump in and out of a browser.

## Usage

Command line arguments for `HTMLtoRSS` are as follows:
```
Usage: HTMLtoRSS [OPTIONS] --html <HTML> --rss <RSS> --base-url <BASE_URL>

Options:
  -l, --html <HTML>                  Path to the HTML file or URL to read
  -r, --rss <RSS>                    Path to the RSS file to update
  -b, --base-url <BASE_URL>          Base URL for converting relative URLs to absolute
  -s, --selector <SELECTOR>          CSS selector to extract HTML content [default: main]
  -t, --title <TITLE>                Title for the RSS item (defaults to first <h1> text)
  -c, --lines-to-cut <LINES_TO_CUT>  Number of lines to cut from the beginning of HTML text (defaults to 0)
      --dry-run                      Dry run mode - only display output to terminal
  -h, --help                         Print help
  -V, --version                      Print version
~/Sites/osola.org.uk $
```

For example, assuming you are in your site root directory and the file to grab the content from is in the `blog` subdirectory and the RSS.xml file is also in the `blog` subdirectory, then run the app like this:

`HTMLtoRSS --html blog/holiday.html --rss blog/rss.xml --base-url https://yoursite.com/blog`

This will create a new RSS item from all the content in the `<main>` element of the local file `blog/holiday.htm` with the title copied from the first `<h1>` element.

Here's another example:

`HTMLtoRSS --html https://yoursite.com/blog/holiday.html --rss blog/rss.xml --base-url https://yoursite.com/blog --title "My Holday in France" --selector body --lines-to-cut 3`

This will create a new RSS item from the `<body>` element of the website page https://yoursite/blog/holiday.html with the title "My Holday in France" and the first 3 content lines removed (perhaps an `<h1>` or other element you didn't want in the feed item).

In both cases, all images, links and other elements with a relative URL will be be converted to absolute URLs, so that (e.g.) an image in the HTML with a `src` attribute value of `images/holiday01.jpg` wil be converted to `https://yoursite.com/blog/images/holiday01.jpg` so that all resources can be viewed externally in the feed reader.

## Build steps

Build the application thus:
* [Install Rust](https://rust-lang.org/tools/install/)
* `git clone https://github.com/bobosola/HTMLtoRSS.git`
* `cd HTMLtoRSS`
* `cargo build --release`

The executable will be in `target/release` as `HTMLtoRSS`. On Windows, the executable will be called `HTMLtoRSS.exe`. You can use PowerShell or another other Windows terminal to run it.

To use the app from anywhere on Macs and Unix machines, I suggest either moving it to `/usr/local/bin` or creating a link to it thus:

* `sudo mv target/release/HTMLtoRSS /usr/local/bin` or
* `sudo ln -s /users/your_home/HTMLtoRSS/target/release/HTMLtoRSS /usr/local/bin/HTMLtoRSS`


## Requirements

You will need a valid `rss.xml` file somewhere locally. You can copy the included demo `rss.xml` file which is a minimal valid RSS.xml file. Change the various values accordingly.

## Creating an RSS feed

1) Populate your `rss.xml` file with items as above and place it on your site.

2) Place a `<link>` element in the `<head>` section of your home page linking to your `rss.xml` file:
```html
<link rel="alternate" type="application/rss+xml" title="RSS Feed for my Blog" href="/blog/rss.xml">
```
This will allow interested people to subscribe to your feed by pasting your home page URL into their feed reader.

Option: insert an RSS logo of your choice in a suitable location which also links to the `rss.xml` file:
```html
<a href="/blog/rss.xml"><img src="/blog/images/rss_logo.gif" alt="RSS logo" width="36" height="14"></a>
```
Back in the day this would cause browsers to open a built-in feed reader, but most modern browsers no longer support this. So clicking the logo will just show the raw feed xml. But experienced RSS enthusiasts know that the linked URL can be copied and pasted into a feed reader to subscribe to the site as an alternative to using the home page URL.
