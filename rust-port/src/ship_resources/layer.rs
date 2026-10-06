#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct Layer(String);

impl Layer {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    pub(crate) fn is_default(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn name(&self) -> &str {
        if self.0.is_empty() {
            "Default"
        } else {
            &self.0
        }
    }
}
