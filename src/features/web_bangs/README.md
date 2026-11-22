# web_bangs
**Does:** configurable aliases (`g`, `yt`, `w` by default; Google is the default search fallback), alias+Space chips (the palette renders them; this module supplies the match), correctly percent-encoded search URLs, empty-bang bookmark suggestions, arbitrary http(s) URL detection (deep paths, queries, ports, fragments), strict validation (http/https only; no user-info, no javascript:/file:/data:), hand-off to the Windows default browser on Enter/click only.
**Does not:** fetch, render, preview or embed any web page; make network calls itself.
Files: `api.rs`, `logic/coordinator.rs` (entry, bang source, URL handler), `bang.rs`, `provider.rs`; URL maths is shared pure code in `core/url.rs`.
