#[derive(Debug, Default, Clone, Copy)]
pub struct TokenTypes {
    pub comment: u32,
    pub number: u32,
    pub string: u32,
    pub pragma_strict: u32,
    pub appendto: u32,
    pub id: u32,
    pub var_scope: u32,
    pub nil: u32,
    pub keyword: u32,
    pub parameter: u32,
    pub method: u32,
    pub parameter_type: u32,
    pub bool: u32,
}
