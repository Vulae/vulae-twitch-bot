
# [vulae-twitch-bot](https://github.com/Vulae/vulae-twitch-bot)

Twitch bot for [twitch.tv/vulae_](https://twitch.tv/vulae_)

## [Commands](#commands)

* [`!github`](./data.yaml) - My GitHub
* [`!bot`](./data.yaml) - Link to this page
* [`!commands`](./data.yaml) - Link to this section of the page
* [`!dotfiles`](./data.yaml) - My ~/.config/
* [Radio](./src/commands/radio.rs)
    * `!song` - Current song URL
    * `!sr [URL]` - Request a song (YouTube only)
    * `!skip` - Skip current song
* [Neovim](./src/commands/neovim.rs)
    * `!theme [theme]` - Set neovim theme (Only for current sessions)

## [TODO](#todo)

* `!wallpaper [URL]` - Set desktop wallpaper (Probably require review from me & only allow imgur, discord, & reddit links)
* Respond to certain messages (Hi, UwU, & some other stuff) either in chat or with sound alert.
* Twitch chat display TUI

## [License](#license)

[MIT No Attribution](./LICENSE)

