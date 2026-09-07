use super::*;
use crate::events::account::*;
use chekov::error::HandleError;
use futures::{FutureExt, future::BoxFuture};

#[derive(chekov::EventHandler, Clone)]
pub struct AccountProjector {
    pub pool: PgPool,
}

#[chekov::event_handler]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    fn handle(&mut self, event: &AccountOpened) -> BoxFuture<'_, Result<(), HandleError>> {
        let event = event.clone();
        let pool = self.pool.acquire();
        async move {
            let p = pool.await.unwrap();
            let _result = AccountRepository::create(&event, p).await;

            Ok(())
        }
        .boxed()
    }
}
