# setup

## 1. install

```bash
cargo install --path .
```

if `xpost` command doesn't work, add `~/.cargo/bin` to PATH:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 2. get api credentials

1. go to https://developer.x.com/en/portal/dashboard
2. create an app (or use existing)
3. navigate to "keys and tokens"
4. generate api key, api secret, access token, access token secret
   - make sure access token has read + write permissions

## 3. configure

```bash
mkdir -p ~/.config/xpost
nano ~/.config/xpost/config.toml
```

add:

```toml
[twitter]
api_key = "your_api_key"
api_secret = "your_api_secret"
access_token = "your_access_token"
access_token_secret = "your_access_token_secret"
```

save (ctrl+x, y, enter)

## 4. run

```bash
xpost
```

ctrl+enter to post, esc to exit

## troubleshooting

if you get permission errors:
```bash
chmod 600 ~/.config/xpost/config.toml
```

if clipboard paste doesn't work (wayland issue), use ctrl+u to upload from file
