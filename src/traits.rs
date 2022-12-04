pub trait Item {
    fn show(&self);
    fn boxed(self) -> Box<Self>;
}
