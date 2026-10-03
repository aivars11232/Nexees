# Hostile help page

Every construct below is active or remote content that a passive offline help
viewer must not run or fetch. The renderer must keep the two safe links and
the bundled image.

<script>alert("script block")</script>

<iframe src="https://example.org/frame"></iframe>

Inline <img src=x onerror="alert('inline')"> and <a href="#" onclick="alert(1)">handler</a>.

[javascript link](javascript:alert(1))
[mixed-case scheme](JaVaScRiPt:alert(2))
[scheme split by a tab](java	script:alert(3))
[data link](data:text/html;base64,PHNjcmlwdD5hbGVydCg0KTwvc2NyaXB0Pg==)
[file link](file:///etc/passwd)
[vbscript link](vbscript:msgbox(5))
[protocol-relative link](//example.org/tracker)

![remote image](https://example.org/tracker.png)
![protocol-relative image](//example.org/tracker.png)
![data image](data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=)

Safe: [next chapter](chapter-2.md), [online manual](https://example.org/manual)
and a bundled image ![diagram](images/diagram.png).
