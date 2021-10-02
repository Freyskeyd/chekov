use super::*;
use crate::events::account::*;
use futures::{future::BoxFuture, FutureExt};

#[derive(chekov::EventHandler, Clone)]
pub struct AccountProjector {
    pub pool: PgPool,
}

#[chekov::event_handler]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    fn handle(&mut self, event: &AccountOpened) -> BoxFuture<Result<(), ()>> {
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
