use tracing::{field::Visit, span, span::Attributes};
use tracing_core::Field;

#[derive(Default, Debug)]
pub(crate) struct AggregateInstanceData {
    aggregate_id: Option<String>,
    correlation_id: Option<String>,
    message: Option<String>,
}

impl AggregateInstanceData {
    pub fn new(attrs: &Attributes<'_>) -> Self {
        let mut span = Self::default();
        attrs.record(&mut span);
        span
    }
}

impl Visit for AggregateInstanceData {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "aggregate_id" => {
                self.aggregate_id = Some(format!("{:?}", value));
            }
            "correlation_id" => {
                self.correlation_id = Some(format!("{:?}", value));
            }
            "message" => {
                self.message = Some(format!("{:?}", value));
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub(crate) struct WakerVisitor {
    id: Option<span::Id>,
    aggregate_id: Option<String>,
}
impl WakerVisitor {
    const WAKE: &'static str = "waker.wake";
    const WAKE_BY_REF: &'static str = "waker.wake_by_ref";
    const CLONE: &'static str = "waker.clone";
    const DROP: &'static str = "waker.drop";
    const TASK_ID_FIELD_NAME: &'static str = "task.id";

    pub(crate) fn result(self) -> Option<span::Id> {
        self.id
    }
}

impl Visit for WakerVisitor {
    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {
        // don't care (yet?)
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == Self::TASK_ID_FIELD_NAME {
            self.id = Some(span::Id::from_u64(value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "aggregate_id" {
            self.aggregate_id = Some(value.to_string());
        }
    }
}
