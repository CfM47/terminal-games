mod ball;
mod breakout;
mod game2048;
mod minesweeper;
mod pong;
mod snake;
mod tetris;

use crate::{game::GameEntry, scores::Ranking};

pub const CATALOG: &[GameEntry] = &[
    GameEntry {
        id: "snake",
        title: "Snake",
        description: "Eat, grow, and don't bite yourself. Gets faster with every bite.",
        controls: "←↑↓→ / wasd move",
        music: None,
        ranking: Ranking::HighestScore,
        launch: || Box::new(snake::Snake::new()),
    },
    GameEntry {
        id: "tetris",
        title: "Tetris",
        description: "Stack falling pieces and clear lines. Every ten lines the drop speeds up.",
        controls: "←→ move · ↑ z rotate · ↓ soft · space drop",
        music: Some(&tetris::MUSIC),
        ranking: Ranking::HighestScore,
        launch: || Box::new(tetris::Tetris::new()),
    },
    GameEntry {
        id: "breakout",
        title: "Breakout",
        description: "Bounce the ball off your paddle and smash the wall. Aim with the paddle's edges.",
        controls: "←→ / ad move · space launch",
        music: None,
        ranking: Ranking::HighestScore,
        launch: || Box::new(breakout::Breakout::new()),
    },
    GameEntry {
        id: "pong",
        title: "Pong",
        description: "Beat the CPU to seven points. Every match you win, it gets faster and sharper.",
        controls: "↑↓ / ws move",
        music: None,
        ranking: Ranking::HighestScore,
        launch: || Box::new(pong::Pong::solo()),
    },
    GameEntry {
        id: "pong-versus",
        title: "Pong · 2 players",
        description: "Two players on one keyboard. First to seven points wins.",
        controls: "P1 w s · P2 ↑ ↓",
        music: None,
        ranking: Ranking::Unranked,
        launch: || Box::new(pong::Pong::versus()),
    },
    GameEntry {
        id: "2048",
        title: "2048",
        description: "Slide the tiles and merge equal numbers. Reach 2048, then keep going.",
        controls: "←↑↓→ / wasd slide",
        music: None,
        ranking: Ranking::HighestScore,
        launch: || Box::new(game2048::Game2048::new()),
    },
    GameEntry {
        id: "minesweeper-beginner",
        title: "Minesweeper · Beginner",
        description: "Clear the 9×9 field with 10 mines. The first dig is always safe.",
        controls: "←↑↓→ move · space dig · f flag · mouse",
        music: None,
        ranking: Ranking::FastestTime,
        launch: || Box::new(minesweeper::Minesweeper::new(&minesweeper::BEGINNER)),
    },
    GameEntry {
        id: "minesweeper-intermediate",
        title: "Minesweeper · Intermediate",
        description: "Clear the 16×16 field with 40 mines. The first dig is always safe.",
        controls: "←↑↓→ move · space dig · f flag · mouse",
        music: None,
        ranking: Ranking::FastestTime,
        launch: || Box::new(minesweeper::Minesweeper::new(&minesweeper::INTERMEDIATE)),
    },
    GameEntry {
        id: "minesweeper-expert",
        title: "Minesweeper · Expert",
        description: "Clear the 30×16 field with 99 mines. The first dig is always safe.",
        controls: "←↑↓→ move · space dig · f flag · mouse",
        music: None,
        ranking: Ranking::FastestTime,
        launch: || Box::new(minesweeper::Minesweeper::new(&minesweeper::EXPERT)),
    },
];
