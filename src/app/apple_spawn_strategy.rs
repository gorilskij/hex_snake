use crate::basic::HexPoint;

#[allow(unused_macros)]
macro_rules! spawn_schedule {
    (@ spawn($h:expr, $v:expr) ) => {
        crate::app::apple_spawn_strategy::AppleSpawn::Spawn(
            crate::basic::HexPoint { h: $h, v: $v }
        )
    };
    (@ wait($t:expr) ) => {
        crate::app::apple_spawn_strategy::AppleSpawn::Wait {
            total: $t,
            current: 0,
        }
    };
    [ $( $action:tt ( $( $inner:tt )* ) ),* $(,)? ] => {
        vec![
            $(
                spawn_schedule!(@ $action( $( $inner )* ))
            ),*
        ]
    };
}

pub enum AppleSpawn {
    Spawn(HexPoint),
    Wait { total: usize, current: usize },
}

pub enum AppleSpawnStrategy {
    None, // no apples
    Random {
        apple_count: usize,
    },
    // a new apple is spawed each time there are not enough apples on the board
    ScheduledOnEat {
        apple_count: usize,
        spawns: Vec<AppleSpawn>,
        next_index: usize,
    },
    // apples are spawned at a given time
    // ScheduledOtTime { .. }
}
