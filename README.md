
# [vulae-twitch-bot](https://github.com/Vulae/vulae-twitch-bot)

Twitch bot for [twitch.tv/vulae_](https://twitch.tv/vulae_)

## [Commands](#commands)

* [`!github`](./src/commands/mod.rs#L15) - My GitHub
* [`!bot`](./src/commands/mod.rs#L15) - Link to this page
* [`!commands`](./src/commands/mod.rs#L15) - Link to this section of the page
* [`!dotfiles`](./src/commands/mod.rs#L15) - My ~/.config/
* [Radio](./src/commands/radio.rs)
    * `!song` - Current song URL
    * `!sr [URL]` - Request a song (YouTube only)
    * `!skip` - Skip current song
* [Neovim](./src/commands/neovim.rs)
    * `!theme [theme]` - Set neovim theme (Only for current sessions)

## [TODO](#todo)

* `!wallpaper [URL]` - Set desktop wallpaper (Probably require review from me & only allow imgur, discord, & reddit links)
* Config file for some stuff
* Respond to certain messages (Hi, UwU, & some other stuff) either in chat or with sound alert.

## [License](#license)

[MIT No Attribution](./LICENSE)

