slotmap::new_key_type! {
    pub struct SourceId;
}

impl SourceId {
    pub fn dummy() -> Self {
        Self(slotmap::KeyData::from_ffi(0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loc {
    pub span: core::ops::Range<usize>,
    pub file: SourceId,
}

impl Loc {
    pub(crate) fn new(span: core::ops::Range<usize>, file: SourceId) -> Self {
        Self { span, file }
    }
}
