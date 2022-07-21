use tokio::sync::mpsc;

use chekov_api as proto;

#[derive(Debug)]
pub(crate) struct Watch<T>(mpsc::Sender<Result<T, tonic::Status>>);

impl<T: Clone> Watch<T> {
    pub(crate) fn update(&self, update: &T) -> bool {
        if let Ok(reserve) = self.0.try_reserve() {
            reserve.send(Ok(update.clone()));
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    Update(Watch<proto::Update>),
}
