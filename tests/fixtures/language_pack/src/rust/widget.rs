use crate::support::RustDependency;

pub const RUST_LIMIT: usize = 4;

pub struct RustWidget {
    value: usize,
}

pub enum RustMode {
    Compact,
}

pub trait RustRenderable {
    fn render(&self);
}

impl RustWidget {
    pub fn build_rust_widget(
        value: usize,
    ) -> Self {
        Self { value }
    }
}

pub fn rust_entrypoint(
    widget: RustWidget,
) -> usize {
    let _ignored = "fn RustStringFake() {}";
    widget.value
}
