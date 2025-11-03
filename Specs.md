# HTMLtoRSS Specifications for LLM

**Comment: This was passed to my local LLM `qwen3-coder-30b-a3b-instruct-mlx` to write the first draft. It did a decent job, however many improvements and change were made afterwards.**

A robust Rust command-line tool for converting a fragment of HTML content to an RSS feed item. It is designed for bloggers who use static HTML pages and want to add RSS feed functionality to their site.

The application arguments should be:
 - the relative path and name of the HTML file to be read, or a URL from which to extact the HTML (by using curl or similar)
 - the relative path and name of the RSS.xml file in which to insert the new item
 - a base URL (e.g. 'https://sitename/blog') to allow any relative URLs in the text to be converted to absolute URLs
 - an optional name of a CSS selector from which to extract the HTML content from the HTML file or URL, should default to "main" if not supplied
 - an optional Title argument for the item Title element, should default to the text from the first `<h1>` element if not supplied
 - an optional lines-to-cut argument which, if supplied, will cut the first n lines of the HTML text to remove any unwanted content
 - an optional dry run mode to display the output to the terminal for testing purposes - NB: only text output is required, no XML or JSON

The application will:
- clean up the extracted HTML text to remove any formatting whitespace
- ensure that the title text is XML-safe
- convert any relative URL in 'src', 'href' and 'srcset' attributes to absolute URLs based on the supplied base URL
- create an RSS Item element from the extracted HTML with the supplied Title and the text as the Description element wrapped inside `![CDATA[ ... ]]` as per the file demo_rss.xml
- The Guid element will be a new Guid
- The date element will be the date and time the application is run in the approved RSS format as per the example file
- the entire item element should be formatted and indented appropropriately to match the existing XML in the rss.xml file
- the completed element should be inserted into the <channel> element of the rss.xml file

A sample.html file is supplied for testing purposes.
