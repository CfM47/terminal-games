# terminal-games

A tiny arcade for your terminal. Pick a game from a `gh`-style menu and play it
with Unicode blocks, colors and NES-style chiptune sound synthesized on the fly —
no audio files involved.

```
                     T E R M I N A L   G A M E S

       ? Choose a game: [Use arrows to move, type to filter]
       > Snake
         Tetris
         Breakout
         Pong
         ...
```

## Games

| Game | Controls | Ranked by |
| --- | --- | --- |
| Snake | `←↑↓→` / `wasd` | score |
| Tetris | `←→` move, `↑` `z` rotate, `↓` soft drop, `space` hard drop | score |
| Breakout | `←→` / `ad` move, `space` launch | score |
| Pong (vs CPU) | `↑↓` / `ws` | score |
| Pong (2 players) | P1 `w` `s`, P2 `↑` `↓` | unranked |
| 2048 | `←↑↓→` / `wasd` | score |
| Minesweeper (beginner, intermediate, expert) | `←↑↓→` move, `space` dig, `f` flag, or the mouse | fastest time |

Everywhere: `p` pauses, `esc` goes back to the menu, `ctrl+c` quits.

High scores are kept in `scores.toml` inside your data directory
(`~/.local/share/terminal-games` on Linux, `%APPDATA%\terminal-games` on Windows,
`~/Library/Application Support/terminal-games` on macOS).

## Install

### Prebuilt binaries

Download the archive for your platform from the
[latest release](https://github.com/CfM47/terminal-games/releases/latest), extract it
and run `tgames`. On Windows, double-clicking `tgames.exe` opens it in a console window.

### From crates.io

```sh
cargo install terminal-games
tgames
```

On Linux, building needs the ALSA development files:

```sh
sudo apt install libasound2-dev   # Debian, Ubuntu
sudo pacman -S alsa-lib           # Arch
sudo dnf install alsa-lib-devel   # Fedora
```

## Terminal tips

- Any modern terminal with Unicode and 256 colors works.
- Breakout and Pong feel smoothest where the terminal reports key releases: always on
  Windows, and on Linux and macOS in terminals with the kitty keyboard protocol
  (kitty, WezTerm, Ghostty, foot, Alacritty).
- If no audio device is available the games simply run silent.

## License

[MIT](LICENSE)
