# xpost

post to x (twitter) from your terminal

## install

```bash
cargo install --path .
```

make sure `~/.cargo/bin` is in your PATH

## setup

1. get api credentials from https://developer.x.com/en/portal/dashboard
   - create an app with read + write permissions
   - grab your api key, api secret, access token, and access token secret

2. create config at `~/.config/xpost/config.toml`:

```toml
[twitter]
api_key = "your_api_key"
api_secret = "your_api_secret"
access_token = "your_access_token"
access_token_secret = "your_access_token_secret"
```

## usage

### post a tweet

just run `xpost` and start typing

**keyboard shortcuts:**
- type to compose (multiline supported)
- ctrl+v - paste image from clipboard
- ctrl+u - upload image from file
- ctrl+p - post
- esc - exit

### view post stats

run `xpost stats` to view statistics for your recent posts

**navigation:**
- ↑/↓ - navigate through your posts
- enter - view detailed stats (likes, retweets, replies, impressions)
- esc - go back / exit
- q - quit

## notes

- character counter shows but doesn't enforce limits (premium accounts work fine)
- supports jpeg, png, gif, webp
- images auto-convert to png on upload
- clipboard paste doesn't work on wayland (use ctrl+u instead)
- config file auto-sets to 600 permissions

## dev

```bash
cargo run
cargo test
```

built with ratatui, crossterm, reqwest, oauth1-request, arboard
