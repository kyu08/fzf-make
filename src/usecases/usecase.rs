pub trait Usecase {
    fn command_str(&self) -> Vec<&'static str>;
    // TODO: Return Result.
    fn run(&self);
}
