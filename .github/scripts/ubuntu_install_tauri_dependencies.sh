#!/bin/sh
set -x
sudo apt-get update
# From https://github.com/tauri-apps/tauri-action/blob/084d50df57534d31b5af88966da31affc0be8ea3/README.md?plain=1#L63
# see more recent versions for updates
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
