# E-Hentai / ExHentai Protocol Reference (for RHenTai)

Sources: JHenTai `c1470866` (`lib/src/network/eh_request.dart`, `lib/src/utils/eh_spider_parser.dart`,
`lib/src/consts/eh_consts.dart`, `lib/src/service/gallery_download/*`), original EhViewer `ba6d41b`
(`client/EhEngine.java`, `client/EhUrl.java`, `client/parser/*`), and live probes run on 2026-10-14
(marked **LIVE**). When live markup and the references differ, live wins. Parsers must tolerate both the
current and the legacy markup where both are described.

## 1. Endpoints

### 1.1 Hosts

| Purpose | EH | EX |
|---|---|---|
| Gallery root | `https://e-hentai.org/` | `https://exhentai.org/` |
| API | `https://api.e-hentai.org/api.php` | `https://s.exhentai.org/api.php` |
| Popular / Watched | `/popular`, `/watched` | same paths |
| Favorites | `/favorites.php` | same |
| Favorite popup | `/gallerypopups.php` | same |
| Toplist | `https://e-hentai.org/toplist.php` (EH only) | — |
| Forums (login) | `https://forums.e-hentai.org/index.php` | — |

### 1.2 List and search query

Query parameters on the root, `/watched` and `/favorites.php`:
- `f_search=<keyword>` (raw text; a tag is written `namespace:"tag$"`, e.g. `female:"glasses$"`).
- `f_cats=<bitmask of EXCLUDED categories>` (omit the parameter, or send 0, for all).
- Advanced: `f_sh=on` (show expunged), `f_sto=on` (require torrent), `f_spf=<min pages>`,
  `f_spt=<max pages>` (send `f_spt` only when it is ≥ `f_spf`), `f_srdd=<min rating 2..5>`. Also available
  but unused: `f_sfl=on`, `f_sfu=on`, `f_sft=on` (disable language, uploader and tag filters).
- Pagination is by cursor: `next=<gid>` or `prev=<gid>`. Cursor values come from the hrefs of `#unext` and
  `#uprev` (regex `next=([\d-]+)` / `prev=([\d-]+)`). When the element is not an `<a>`, or has no href,
  that direction has ended. `seek=YYYY-MM-DD` jumps by date (optional).
- Favorites: `favcat=<0..9>` (omit for all), plus `f_search` and the `next`/`prev` cursors.
- Toplist: `tl=15` (day), `13` (month), `12` (year), `11` (all), and `p=<0-based page>`.
- `/popular` is a single page with no cursor.

### 1.3 Category bits (`f_cats`, excluded categories)

Misc 1, Doujinshi 2, Manga 4, Artist CG 8, Game CG 16, Image Set 32, Cosplay 64, Asian Porn 128,
Non-H 256, Western 512.

Site spellings to accept when parsing: `Doujinshi`, `Manga`, `Artist CG`, `Game CG`, `Western`,
`Non-H`, `Image Set`, `Cosplay`, `Asian Porn`, `Misc`, `Private`. The gdata API uses the same display
strings, e.g. `"Misc"`.

### 1.4 Gallery detail

- `GET /g/{gid}/{token}/?p={n}`. `p` is the 0-based thumbnail page; add `hc=1` to show all comments.
- Content warnings: the client sets cookie `nw=1` on `.e-hentai.org` and `.exhentai.org` at startup and
  after logout (JHenTai does the same). A response that still contains `<h1>Content Warning</h1>` is
  retried once with `?nw=session`.
- **LIVE** wrong token: a 39-byte body, `Key missing, or incorrect key provided.` → `notFound`.

### 1.5 Image page

- `GET /s/{page_token}/{gid}-{index+1}`; on retry, add `?nl=<reload_key>`. Send Referer = the gallery URL.

### 1.6 `api.php` (POST JSON, `Content-Type: application/json`)

- `gdata`: `{"method":"gdata","gidlist":[[gid,"token"],…],"namespace":1}`. At most **25** pairs per
  call. **LIVE** response: `{"gmetadata":[{gid, token, title, title_jpn, category, thumb, uploader,
  posted:"<unix secs as string>", filecount:"<string>", filesize:<number>, expunged:<bool>,
  rating:"3.70", torrentcount:"0", torrents:[…], tags:["parody:xxx","female:yyy", "zzz"]}]}`. An entry
  that failed has `{"gid":…, "error":"…"}`. Numbers often arrive as strings, so parse leniently.
- `gtoken`: `{"method":"gtoken","pagelist":[[gid,"gallery_token",page_1_based]]}` → `tokenlist[0].token`.
- `showpage`: `{"method":"showpage","gid":gid,"page":p1,"imgkey":page_token,"showkey":showkey}`. The
  response fields `i3` (image HTML), `i6` (`nl(...)`) and `i7` (original link) are optional
  optimizations; RHenTai may always fetch `/s/` HTML instead.
- `rategallery`: `{"method":"rategallery","apiuid":uid,"apikey":key,"gid":gid,"token":t,"rating":1..10}`
  (not in MVP).

### 1.7 Favorites mutations

- Single gallery: `POST /gallerypopups.php?gid={gid}&t={token}&act=addfav` (form-urlencoded).
  - Add or move: `favcat=<0..9>&favnote=<note>&apply=Add to Favorites&update=1`
  - Remove: `favcat=favdel&favnote=&apply=Apply Changes&update=1`
- Batch (on the favorites page URL): `POST /favorites.php` with `ddact=fav<0..9>` or `ddact=delete`,
  repeated `modifygids[]=<gid>`, and `apply=Apply`.
- Success is HTTP 200 without an error box. Afterwards, re-read the list or detail for the new state.
- Reading the current favorite: `GET /gallerypopups.php?gid={gid}&t={token}&act=addfav` (paced like any HTML
  request, Referer = the gallery URL). The form `#galpop` holds radio inputs `favcat` 0..9 (ids `fav0`..`fav9`)
  and `textarea[name=favnote]`, which contains the user's current note (empty when not favorited). **LIVE
  2026-10-15**: one radio is ALWAYS `checked`, and `fav0` is checked even when the gallery is not favorited, so
  never infer the favorited state or slot from the popup; take them from the detail page. When favorited, the form
  also offers `favdel`. This popup is the ONLY place
  the note can be read: the detail page and the lists show the category name, never the note.

## 2. Login and session

### 2.1 Cookies

| Cookie | Domain | Meaning |
|---|---|---|
| `ipb_member_id` | `.e-hentai.org` **and** `.exhentai.org` | account id (numeric) |
| `ipb_pass_hash` | both | session secret (32 hex) |
| `igneous` | `.exhentai.org` | EX access; invalid if empty, `mystery`, `deleted` or `null` |
| `sk` | `.e-hentai.org` | settings/profile key set by the site (pass through) |
| `nw` | both | `1` = skip content warnings (set by the client) |

After any login, copy `ipb_member_id` and `ipb_pass_hash` to `.exhentai.org`, then `GET https://exhentai.org/`
to obtain `igneous`.

### 2.2 Login-state check (**LIVE**)

`GET https://e-hentai.org/home.php` without following redirects. A `302` to `…/bounce_login.php?…` means
not logged in. A `200` means logged in. `favorites.php` behaves the same way, so any 302 to
`bounce_login.php` maps to `loginRequired`.

### 2.3 Password login

`POST https://forums.e-hentai.org/index.php?act=Login&CODE=01` (form) with
`referer=https://forums.e-hentai.org/index.php&b=&bt=&UserName=<u>&PassWord=<p>&CookieDate=365`.
Headers: `Origin: https://forums.e-hentai.org`, `Referer: https://forums.e-hentai.org/index.php?act=Login&CODE=00`.
- Success: the response sets both `ipb_member_id` and `ipb_pass_hash`.
- Failure: the body contains `The captcha was not entered correctly` (→ `loginFailed`, "captcha
  required, use web login"), or `<h4>The error returned was:</h4>\s*<p>(.+?)</p>`, or
  `<span class="postcolor">(.+?)</span>` (→ `loginFailed` with that text).
- **LIVE 2026-10-14:** forums.e-hentai.org answers non-browser HTTP clients with a Cloudflare challenge:
  status 403, header `cf-mitigated: challenge`, title `Just a moment...`. Map this to `cloudflare` with a
  message recommending web login or cookie login. Password login is therefore best-effort only.

### 2.4 Web login (desktop, recommended)

Open a real webview on `https://forums.e-hentai.org/index.php?act=Login&CODE=00`. The user passes the
Cloudflare check, any captcha, and the form. Read the cookies through the webview cookie API, never
through `document.cookie`. Then navigate to `https://exhentai.org/` so the site's redirect chain (**LIVE**:
302 to `https://forums.e-hentai.org/remoteapi.php?ex=<token>`) can issue `igneous`. Import every
cookie for `e-hentai.org`, `forums.e-hentai.org` and `exhentai.org` into the client jar.

### 2.5 Cookie login

The user pastes `ipb_member_id`, `ipb_pass_hash` and optionally `igneous`. Store them for both domains,
verify with §2.2, then try §2.1's EX request. If EX access cannot be obtained (sad panda, or Cloudflare
on the redirect), keep the login and set `ex_access=false`.

### 2.6 ExHentai checks

A sad panda is a `200` with an empty body (or only a tiny image) from `exhentai.org`; map it to
`noExAccess`. With no EX cookies at all, the **LIVE** behaviour is a 302 to `forums.e-hentai.org/remoteapi.php`.

### 2.7 Username (best effort)

`GET https://forums.e-hentai.org/index.php?showuser=<member_id>` → `#profilename`. This may be
Cloudflare-blocked; on failure, leave `username = None`.

### 2.8 Logout

Clear the jar, re-add `nw=1`, and persist.

## 3. HTML parsing

### 3.1 Gallery lists (any display mode)

- Root: the first `.itg` element. Mode classes are `gltm` (minimal), `gltc` (compact, **LIVE** default),
  `glte` (extended), and `gld` (thumbnail grid, `div` children).
- Identity: every `a[href]` inside `.itg` matching `/g/(\d+)/([0-9a-f]{10})/`, de-duplicated by gid in
  document order.
- Fallback per-item fields, scoped to the row (`tr` or `.gl1t`) that contains the anchor:
  title `.glink`; category `.cn` / `.cs` text; thumbnail `img` `data-src` then `src`; rating from the
  `.ir` style `background-position:-Xpx -Ypx` as `5 - X/16 - (Y == 21 ? 0.5 : 0)`; posted from the
  `[id^=posted_]` text `YYYY-MM-DD HH:MM` (UTC).
- Favorite marker: `#posted_<gid>`. A `title` attribute holds the favorite category name, and the style
  `border-color:#xxx` maps to the slot through this palette: `#000`→0, `#f00`→1, `#fa0`→2, `#dd0`→3,
  `#080`→4, `#9f4`→5, `#4bf`→6, `#00f`→7, `#508`→8, `#e8e`→9. No title or border means not favorited.
- Empty result: the page contains `No hits found` → empty items (not an error). `/watched` with no
  watched tags is also empty.

### 3.2 Gallery detail

- Title `#gn`; Japanese title `#gj` (empty means None); category `#gdc .cs` (or `#gdc div`); uploader
  `#gdn a` (or `#gdn` text); cover `#gd1 > div` style `url(...)`.
- `#gdd tr`: `.gdt1` label → `.gdt2` value. Labels are `Posted:` (`YYYY-MM-DD HH:MM`, UTC), `Parent:` (an
  `a[href]` to `/g/…/` or `None`), `Visible:`, `Language:` (strip trailing `&nbsp; TR`), `File Size:`,
  `Length:` (`87 pages`), `Favorited:` (`123 times`, `Once` or `Never`).
- Rating: `#rating_label` (`Average: 3.70`, or `Not Yet Rated`); `#rating_count`.
- Tags: `#taglist tr`. `td.tc` is the namespace with its trailing `:`; tags are `td:nth-child(2) a`
  (text). Element ids look like `ta_<namespace>:<tag_with_underscores>`.
- Favorite: `#favoritelink` text (`Add to Favorites` means not favorited; otherwise the category name),
  and `#fav .i` style `background-position:0px -Ypx` → slot `(Y - 2) / 19`. The `#fav .i` `title` attribute,
  when present, is also the category name. It is NOT the user's favorite note: never map it to
  `favorite_note`. The note comes only from the popup (§1.7).
- Comments: `#cdiv .c1` → author `.c3 a` (or `.c3` text after `by:`), posted text from `.c3`
  (`Posted on …`), score `.c5 span`, body `.c6` (text, `<br>` → newline). The uploader comment has `#comment_0`.
- API credentials (internal): `var apiuid = (\d+);` and `var apikey = "(\w+)";`.

### 3.3 Detail thumbnails (**LIVE** markup)

```html
<div id="gdt" class="gt200">
  <a href="https://e-hentai.org/s/fb698736e7/4253531-1">
    <div title="Page 1: 1_1.jpg"
         style="width:200px;height:300px;background:transparent url(https://…/4253531-0.webp) -0px 0 no-repeat"></div>
  </a> …
```

- The class is `gt100` (normal) or `gt200` (large); both use sprite sheets with background offsets.
- Style regex: `width:(\d+)px;height:(\d+)px;background:transparent url\(([^)]+)\) (-?\d+)(?:px)? (-?\d+)(?:px)?`.
- Page token and index come from the href: `/s/([0-9a-f]{10})/(\d+)-(\d+)` (1-based index → subtract 1).
- File name comes from the title: `^Page \d+: (.+)$`.
- Legacy markup fallback: `.gdtm` / `.gdtl` children, with an `img` `src` instead of a sprite (offset 0).
- Page count: the maximum integer among `.ptt td a` texts (default 1). **LIVE**: 20 thumbs per page at
  `gt200`. Also present: `.gpc` text `Showing 1 - 20 of 87 images`.

### 3.4 Image page (**LIVE** markup)

```html
<img id="img" src="https://xhh….hath.network:8443/h/…/1_1.webp" style="width:1280px;height:1920px"
     onerror="this.onerror=null; nl('44130-497768')" …>
```

- Image URL: `#img` `src`. Size: the style `width:(\d+)px` and `height:(\d+)px`.
- Reload key: regex `nl\('([^']+)'\)` (in `#img` `onerror` and in `#loadfail` `onclick`).
- `var showkey="([0-9a-z]+)"`. Page counter: `#i2 .sn div` (`1 / 87`).
- File info: `#i4 > div` text, e.g. `1_1.webp :: 1280 x 1920 :: 167.4 KiB`; file name is the text before ` :: `.
- Original image (logged in only): `#i6 a[href*="fullimg"]` (older markup: `fullimg.php`).
- 509: an image URL ending in `/509.gif` (`https://ehgt.org/g/509.gif`, `https://exhentai.org/img/509.gif`) → `imageLimit`.

### 3.5 Favorites page (login required)

- Categories: `.nosel .fp` (or `.ido .fp`). There are 10 entries for slots 0-9, then one extra `fp fps`
  entry for "all", which is skipped. Count is in `div:first-child`, name in `div:last-child` (EhViewer: child
  0 and child 2).
- Items: same as §3.1. Each item's slot comes from `#posted_<gid>`.
- Capture a fixture after the first real login and add a test.

## 4. Special pages and errors (check before parsing)

| Condition | Detection | ErrorKind |
|---|---|---|
| Cloudflare challenge | status 403/503 **and** (`cf-mitigated: challenge` header **or** title `Just a moment...`) | `cloudflare` |
| IP ban | body starts with `Your IP address has been temporarily banned` or `This IP address has been temporarily banned` | `banned` (message = full text with expiry) |
| Login redirect | 302 to `bounce_login.php` | `loginRequired` |
| Sad panda | `exhentai.org` 200 with empty body | `noExAccess` |
| Wrong token | `Key missing, or incorrect key provided.` | `notFound` |
| Removed or unavailable | `.d p` text `This gallery has been removed…`, `This gallery is unavailable due to a copyright claim…`, `Gallery not found.`, `This gallery is pining for the fjords.` | `notFound` (message = text) |
| Image limit | `You have exceeded your image viewing limits` or a 509 image URL | `imageLimit` |
| Fatal server error | `Page load has been aborted due to a fatal error` | `http` |
| Non-2xx (other) | — | `http` (message includes status) |
| Transport or timeout | reqwest error | `network` |

## 5. Practices

- User-Agent: a current desktop Chrome UA. Use one constant in `eh-client`, overridable in config.
- Pacing: at most one HTML or API request every 350 ms per client (global), with ≤ 3 in flight.
  Images are not paced but are bounded by the download/reader semaphores.
- Download defaults: concurrency 4 (JHenTai uses 6 with 2 task starts per second), up to 5 attempts per
  page, re-resolve with `nl` after a failure, and treat HTTP 403 on an image as a dead H@H node
  (re-resolve immediately).
- Original images: only when enabled and logged in. Fall back to the resampled image when the
  `fullimg` link is missing.
- Folder naming follows JHenTai's `<gid> - <sanitized title>`; RHenTai uses 1-based zero-padded page
  files and `rhentai.json` metadata (ARCHITECTURE §7).
