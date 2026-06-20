#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Precedence {
    None,
    Or,
    And,
    Comparison,
    Term,
    Factor,
    Power,
    Unary,
    Call,
}
