# stats mode

view engagement metrics for your posts

## usage

```bash
xpost stats
```

shows your last 20 posts with:
- likes
- retweets
- replies (count)
- quotes
- impressions (if you have elevated api access)

## navigation

- `↑/↓` - browse posts
- `enter` - view detailed stats
- `esc` - go back
- `q` - quit

## notes

- impressions require elevated api access (basic tier shows 0)
- uses same credentials from `~/.config/xpost/config.toml`
- fetches from twitter api v2

## troubleshooting

if stats don't load:
- make sure your app has read permissions at https://developer.x.com/en/portal/dashboard
- check you haven't hit rate limits (twitter caps api calls)
