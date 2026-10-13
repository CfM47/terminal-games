mod snake;

use crate::game::GameEntry;

pub const CATALOG: &[GameEntry] = &[GameEntry {
    id: "snake",
    title: "Snake",
    description: "Eat, grow, and don't bite yourself. Gets faster with every bite.",
    controls: "←↑↓→ / wasd move",
    launch: || Box::new(snake::Snake::new()),
}];
