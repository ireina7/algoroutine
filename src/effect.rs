/// Parameters have initial object
pub trait Param {
    fn initial() -> Self;
}

pub trait View<A, B> {
    fn view(self) -> Result<A, B>;
}

// TODO Are you sure that each effect only has one return type?
pub trait Effect {
    type Return;
}
