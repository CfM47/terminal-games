mod ball;
mod breakout;
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
        launch: || Box::new(snake::Snake::new()),
    },
    GameEntry {
        id: "tetris",
        title: "Tetris",
        description: "Stack falling pieces and clear lines. Every ten lines the drop speeds up.",
        controls: "←→ move · ↑ z rotate · ↓ soft · space drop",
        music: Some(&tetris::MUSIC),
        launch: || Box::new(tetris::Tetris::new()),
    },
    GameEntry {
        id: "breakout",
        title: "Breakout",
        description: "Bounce the ball off your paddle and smash the wall. Aim with the paddle's edges.",
        controls: "←→ / ad move · space launch",
        music: None,
        launch: || Box::new(breakout::Breakout::new()),
    },
];
