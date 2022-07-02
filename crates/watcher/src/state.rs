#[derive(Default, Debug)]
pub(crate) struct State {
    // metas: HashMap<u64, Metadata>,
    // last_updated_at: Option<SystemTime>,
    // temporality: Temporality,
    // tasks_state: TasksState,
    // resources_state: ResourcesState,
    // async_ops_state: AsyncOpsState,
    // current_task_details: DetailsRef,
    // retain_for: Option<Duration>,
    // strings: intern::Strings,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) fn update(&mut self, _: ()) {}
}
