use std::write;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Default)]
pub enum MarginalFitness
{
    Scheduled(u64),
    #[default]
    None,
}

impl std::fmt::Debug for MarginalFitness
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            MarginalFitness::Scheduled(time) => write!(
                f,
                "{}::Scheduled({}, {:?}, {:?})",
                std::any::type_name::<MarginalFitness>()
                    .split("::")
                    .last()
                    .unwrap(),
                time,
                time / 3600,
                time / 3600 / 24,
            ),
            MarginalFitness::None => write!(f, "MarginalFitness::None"),
        }
    }
}
