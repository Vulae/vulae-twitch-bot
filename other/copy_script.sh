#!/bin/bash

SCRIPT_DIR="$(dirname "$(realpath "$0")")"
cd "$SCRIPT_DIR" || exit 1

cp ~/.config/nvim/lua/vulae_twitch_bot.lua ./nvim/lua/vulae_twitch_bot.lua
