# archive-org-xboxcds-torrents

[Archive.org](https://archive.org) offers links to download large collections
of archived goods.

This little application makes it easy to download the whole
[xboxcds collection](https://archive.org/details/xboxcds).

```sh
cargo run | xargs -n 1 curl -sO
```
