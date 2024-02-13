use super::*;
use crate::snake::eat_mechanics::EatMechanics;
use std::fmt::{Display, Formatter};

#[derive(Debug, Error)]
#[must_use]
pub struct BuilderError(pub Box<Builder>, pub &'static str);

impl Display for BuilderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "snake builder error: {}", self.1)?;
        writeln!(f, "builder: {:?}", self.1)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Builder {
    pub snake_type: Option<Type>,
    pub eat_mechanics: Option<EatMechanics>,

    pub pos: Option<HexPoint>,
    pub dir: Option<Dir>,
    pub len: Option<usize>,
    pub speed: Option<f32>,

    pub palette: Option<PaletteTemplate>,
    pub controller: Option<snake_control::Template>,

    pub autopilot: Option<pathfinder::Template>,
    pub autopilot_control: bool,
}

impl Builder {
    #[inline(always)]
    #[must_use]
    pub fn snake_type(mut self, value: Type) -> Self {
        self.snake_type = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn eat_mechanics(mut self, value: EatMechanics) -> Self {
        self.eat_mechanics = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn pos(mut self, value: HexPoint) -> Self {
        self.pos = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn dir(mut self, value: Dir) -> Self {
        self.dir = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn len(mut self, value: usize) -> Self {
        self.len = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn speed(mut self, value: f32) -> Self {
        self.speed = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn palette(mut self, value: PaletteTemplate) -> Self {
        self.palette = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn controller(mut self, value: snake_control::Template) -> Self {
        self.controller = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn autopilot(mut self, value: pathfinder::Template) -> Self {
        self.autopilot = Some(value);
        self
    }

    #[inline(always)]
    #[must_use]
    pub fn autopilot_control(mut self, value: bool) -> Self {
        self.autopilot_control = value;
        self
    }

    pub fn build(&self) -> Result<Snake, BuilderError> {
        let pos = self
            .pos
            .ok_or_else(|| BuilderError(Box::new(self.clone()), "missing field `pos`"))?;
        let dir = self
            .dir
            .ok_or_else(|| BuilderError(Box::new(self.clone()), "missing field `dir`"))?;

        if self.autopilot_control && self.autopilot.is_none() {
            return Err(BuilderError(
                Box::new(self.clone()),
                "autopilot_control set to true but autopilot missing",
            ));
        }

        eprintln!(
            "spawn snake at {:?} coming from {:?} going to {:?}",
            pos, -dir, dir
        );

        let head = Segment {
            segment_type: SegmentType::Normal,
            pos,
            coming_from: -dir,
            teleported: None,
            z_index: 0,
        };

        let mut cells = VecDeque::new();
        cells.push_back(head);

        let body = Body {
            segments: cells,
            missing_front: 0,
            dir,
            turn_start: None,
            dir_grace: false,
            grow: self
                .len
                .ok_or_else(|| BuilderError(Box::new(self.clone()), "missing field `len`"))?,
            search_trace: None,
        };

        Ok(Snake {
            snake_type: self.snake_type.ok_or_else(|| {
                BuilderError(Box::new(self.clone()), "missing field `snake_type`")
            })?,
            eat_mechanics: *self.eat_mechanics.as_ref().ok_or_else(|| {
                BuilderError(Box::new(self.clone()), "missing field `eat_mechanics`")
            })?,
            speed: self
                .speed
                .ok_or_else(|| BuilderError(Box::new(self.clone()), "missing field `speed`"))?,
            body,
            state: State::Living,
            controller: self
                .controller
                .as_ref()
                .ok_or_else(|| {
                    BuilderError(Box::new(self.clone()), "mssing field `snake_control`")
                })?
                .clone()
                .into_controller(dir),
            palette: self
                .palette
                .ok_or_else(|| BuilderError(Box::new(self.clone()), "mssing field `palette`"))?
                .into(),
            autopilot: self.autopilot.clone().map(|template| {
                let controller_template = snake_control::Template::Algorithm(template);
                controller_template.into_controller(dir)
            }),
            autopilot_control: self.autopilot_control,
        })
    }
}
