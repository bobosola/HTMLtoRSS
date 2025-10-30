# HTML to RSS

This is a command line application to create an RSS feed item from a local static HTML page and insert it into an RSS.xml file. It is designed for occasional bloggers who still use static HTML pages and who wish to add an RSS feed top their site. There is also a demo RSS.xml file with instructions below for setting up an RSS feed from scratch.

The application does the following:
* grabs all the text inside a nominated element
* optionally ignores a number of lines to allow for the removal of unwanted headings etc.
* converts all `href`, `src`, and `srcset` attributes to absolute URLs so that they will work in an external feed reader
* removes all extraneous white space used for formatting
* copies the result into the RSS.xml file as a new feed item with the date and time set to the time of insertion

## Output

The application produces a populated `<item>` element like this :

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

It may seem unintuitive to put the entire page content into the `<description>` element, but it's common practice, and is allowed in the [RSS 2.0 Specifications](https://www.rssboard.org/rss-specification#hrelementsOfLtitemgt). This approach has the advantage of allowing people to read successive articles in their entirety in a feed reader without having to jump in and out of a browser.

## Build steps

The following three constants must be edited accordingly in `src/main.rs`:
* `BASE_URL` - the URL for converting relative paths to full URLs
* `CONTENT_ELEMENT` - the element which contains the content you wish to export to RSS
* `LINES_TO_CUT` - the number of lines to ignore from the start of the chosen element

Then build the application thus:
* [Install Rust](https://rust-lang.org/tools/install/)
* `git clone https://github.com/bobosola/HTMLtoRSS.git`
* `cd HTMLtoRSS`
* `cargo build --release`

The executable will be in `target/release` as `HTMLtoRSS`. On Windows, the executable will be called `HTMLtoRSS.exe`. You can use PowerShell or another other Windows terminal to run it.

To use the app from anywhere on Macs and Unix machines, I suggest either moving it to `/usr/local/bin` or creating a link to it thus:

* `sudo mv target/release/HTMLtoRSS /usr/local/bin` or
* `sudo ln -s /users/your_home/HTMLtoRSS/target/release/HTMLtoRSS /usr/local/bin/HTMLtoRSS`



## Usage

The application requires three arguments:
* an HTML file path relative to the current directory
* the title text for the item as it will appear in a feed reader
* the RSS.xml file relative to the current directory

For example, assuming you are in your site root directory and the file to grab the content from is in the `blog` subdirectory and the RSS.xml file is also in the `blog` subdirectory, then run the app like this:

`HTMLtoRSS blog/my_latest_article.htm "My article title" blog/rss.xml`

The application will confirm success on insertion of the new item.

## Requirements

You will need a valid `rss.xml` file somewhere on your site. You can copy the included demo `rss.xml` file which is a minimal valid RSS.xml file. Change the various values accordingly.

You should also place a `<link>` element in the `<head>` section of your home page linking to your `rss.xml` file:
```html
<link rel="alternate" type="application/rss+xml" title="RSS Feed for my Blog" href="/blog/rss.xml">
```
This will allow interested people to subscribe to your feed by pasting your home page URL into their feed reader.

Another option is to insert an RSS logo of your choice in a suitable location which also links to the `rss.xml` file:
```html
<a href="/blog/rss.xml"><img src="/blog/images/rss_logo.gif" alt="RSS logo" width="36" height="14"></a>
```
Back in the day this would cause browsers to open a built-in feed reader, but nearly all modern browsers no longer support this. So clicking the logo will just show the raw feed xml. But the linked URL can also be used to paste into a feed reader to subscribe to the site.
