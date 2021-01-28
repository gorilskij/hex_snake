use ggez::mint::Point2;

#[derive(Copy, Clone, Debug, Add, AddAssign, Sub, SubAssign)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for Point2<f32> {
    fn from(Point { x, y }: Point) -> Self {
        Point2 { x, y }
    }
}

impl From<Point2<f32>> for Point {
    fn from(Point2 { x, y }: Point2<f32>) -> Self {
        Point { x, y }
    }
}
