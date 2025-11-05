# [Try it for yourself!](https://topdoggo.app)

## Features:
- [Add your dog](https://topdoggo.app/upload)
- Earn XP and level up from voting, and [save progress between devices](https://topdoggo.app/me)
- Check out the [overall leaderboard](https://topdoggo.app/leaderboard/top/overall) and your own [personal leaderboard](https://topdoggo.app/leaderboard/top/personal)

## Techy details:
- Uses the [Elo Rating System](https://en.wikipedia.org/wiki/Elo_rating_system#Theory) (most notably used in competitive chess) to adjust ratings after each vote
- Using HTMX for a minimal javascript bundle (~42kb gzipped) and streamlined DX (single source of truth, no client-side state)
- Fully self-hosted
    - on a VPS using with docker (with a multi-stage build for a final binary size of <20MB)
    - using Plausible on the same VPS (with a reverse proxy) for analytics
- Mobile-friendly styling with dark/light mode and animations using the View Transition API
- Self-rolled magic link passwordless auth
- Notifies me over text when someone uploads a dog for me to approve
