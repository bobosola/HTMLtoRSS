# feeder

Gets HTML text from a web page and cleans it up ready for pasting into an rss.xml file for use in an RSS feed thus:

```
<item>
    <description>
        <![CDATA[
            ... pasted text goes here
        ]]>
    </description>
</item>
```

* Grabs all text inside `<main>` and `</main>` elements
* ignores the first three lines (which are just `H1` and `H2` tags for [https://osola.org.uk/blog](https://osola.org.uk/blog) entries)
* converts all relative paths for links and images to full paths to work in a feed reader
* removes all extraneous white space
* optionally copies the result to clipboard for pasting into the CDATA element

## Usage

* `feeder <html-file-path>` to print to std out
* `feeder <html-file-path> --copy` to copy output to clipboard
