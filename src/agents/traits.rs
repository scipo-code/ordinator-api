/// What should an algorithm be able to do? This is the trait that all scheduling algorithms should
/// implement. It is a trait so that we can have multiple algorithms in the same system. 
pub trait Algorithm {
    fn update_state<T>(&mut self, message: impl actix::Message<T>);

    fn schedule(&mut self);

    fn unschedule(&mut self);

    fn 
}
