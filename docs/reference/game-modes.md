# Game modes and apples

## Modes (`app/game_mode.rs`)

Picked on the start screen (not saved); it sets each player seed's starvation when the game
starts.

| mode | apples | snakes |
|---|---|---|
| **Classic** | `Eat(food)`: eaten and digested — the eaten segment travels down the body and grows the snake by `food` as it reaches the tail ([digestion](snake-model.md#digestion-classic)); an eaten segment can be passed through until digested | never shrink on their own |
| **Hunger** | `Grow(1)`: grows the snake right away, eased out over 1 s (at first faster than the head, so the tail moves backwards); bad apples `Shrink(3–5)` over 1 s | apple-eating snakes shrink steadily (`STARVATION` = 0.3 cells/s); the player starving to `MIN_LENGTH` (1 cell) is game over (`State::Starved`) |

Hunger tuning (`hunger` module): `GROW` = 1 cell over `GROW_DURATION` = 1 s;
`BAD_APPLE_SHRINK` = 3–5 cells over `SHRINK_DURATION` = 1 s; bad apples appear at random,
`BAD_APPLE_INTERVAL` = 10 s on average, at most `MAX_BAD_APPLES` = 3, each disappearing after
`BAD_APPLE_LIFETIME` = 5 s of game time. Expiring apples do not count towards the apple limit.

## Apple types (`apple/mod.rs`)

| type | effect |
|---|---|
| `Eat(food)` | Classic food; `food` is the apple food setting (debug keys `1`–`9`) |
| `Grow(amount)` | Hunger food |
| `Shrink(amount)` | Hunger's bad apples |
| `SpawnSnake(builder)` | a special apple that spawns an AI snake: a **competitor** (an apple seeker, 1 target) or a **killer** (hunts the player); each lives 200 cells (one is counted down per cell it enters), then dies |
| `SpawnRain` | a special apple that makes snakes rain down |

**Special apples** (debug key `X`) replace some food apples, with probabilities from the prefs
(defaults: `prob_spawn_competitor` 0.025, `prob_spawn_killer` 0.015, `prob_spawn_rain` 0.002 per
apple); AI snakes
that run out of life, and rain reaching the bottom row, sink into a hole. Spawning (`apple/spawn.rs`) avoids
occupied cells; apples a tail grows over are moved.
