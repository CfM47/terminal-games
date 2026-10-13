mod ball;
mod breakout;
mod pong;
mod snake;
mod tetris;

use crate::game::GameEntry;

pub const CATALOG: &[GameEntry] = &[
    GameEntry {
        id: "snake",
        title: "Snake",
        description: "Eat, grow, and don't bite yourself. Gets faster with every bite.",
        controls: "←↑↓→ / wasd move",
        music: None,
        ranked: true,
        launch: || Box::new(snake::Snake::new()),
    },
    GameEntry {
        id: "tetris",
        title: "Tetris",
        description: "Stack falling pieces and clear lines. Every ten lines the drop speeds up.",
        controls: "←→ move · ↑ z rotate · ↓ soft · space drop",
        music: Some(&tetris::MUSIC),
        ranked: true,
        launch: || Box::new(tetris::Tetris::new()),
    },
    GameEntry {
        id: "breakout",
        title: "Breakout",
        description: "Bounce the ball off your paddle and smash the wall. Aim with the paddle's edges.",
        controls: "←→ / ad move · space launch",
        music: None,
        ranked: true,
        launch: || Box::new(breakout::Breakout::new()),
    },
    GameEntry {
        id: "pong",
        title: "Pong",
        description: "Beat the CPU to seven points. Every match you win, it gets faster and sharper.",
        controls: "↑↓ / ws move",
        music: None,
        ranked: true,
        launch: || Box::new(pong::Pong::solo()),
    },
    GameEntry {
        id: "pong-versus",
        title: "Pong · 2 players",
        description: "Two players on one keyboard. First to seven points wins.",
        controls: "P1 w s · P2 ↑ ↓",
        music: None,
        ranked: false,
        launch: || Box::new(pong::Pong::versus()),
    },
];
