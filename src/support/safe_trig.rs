pub trait SafeTrig {
    fn safe_asin(self) -> Self;
    fn safe_acos(self) -> Self;
}

impl SafeTrig for f32 {
    fn safe_asin(self) -> Self {
        self.clamp(-1.0, 1.0).asin()
    }

    fn safe_acos(self) -> Self {
        self.clamp(-1.0, 1.0).acos()
    }
}

impl SafeTrig for f64 {
    fn safe_asin(self) -> Self {
        self.clamp(-1.0, 1.0).asin()
    }

    fn safe_acos(self) -> Self {
        self.clamp(-1.0, 1.0).acos()
    }
}
