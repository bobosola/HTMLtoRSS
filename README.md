# HTMLtoRSS

Rough and ready text grabber largely coded by `qwen3-30b-a3b-instruct-2507-mlx`. It pulls content from a local file system HTML page and prepares it for pasting into an RSS XML feed file thus:

```xml
<item>
    <title>{title text}</title>
    <link>{url to web page}</link>
    <description>
        <![CDATA[
            {pasted text from HTMLtoRSS goes here}
        ]]>
    </description>
    <pubDate>Wed, 22 Jul 2025 22:30:00 GMT</pubDate>
    <guid>{any unique string such as url to webpage}</guid>
</item>
```

This is a highly specific tool written to convert blog pages on my own site to enable the page content to be easily pasted into an `rss.xml` file. It does the following:

* grabs all the text inside the `<main>` and `</main>` elements
* ignores the first three lines (which are unwanted `H1` and `H2` tags)
* converts all relative paths (i.e. anything with relative `href`, `src`, or `srcset` attributes) to full URL paths so that they will work in an external feed reader
* removes all extraneous white space used for formatting
* optionally copies the result to clipboard for pasting directly into the `CDATA` element of an `rss.xml` file as above

The following constants are set in `src/main.rs`:
* the base URL for the full path
* the number of lines to cut

## To Build

* install Rust
* `git clone https://github.com/bobosola/HTMLtoRSS.git`
* `cd HTMLtoRSS`
* `cargo build --release`

The executable will be in `/target/release` as `HTMLtoRSS`.

## Usage

* `HTMLtoRSS <html-file-path>` to print to stdout
* `HTMLtoRSS <html-file-path> --copy` to copy the output to clipboard for direct pasting into the `rss.xml` file

You could make an alias to it in `/usr/local/bin` so that you could call it from anywhere:
`sudo ln -s /your/full/path/to/HTMLtoRSS/target/release/HTMLtoRSS /usr/local/bin/HTMLtoRSS`

Then you could use it like this:
`HTMLtoRSS path/to/yourfile.htm --copy`
