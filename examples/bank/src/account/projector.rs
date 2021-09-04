use super::*;
use futures::{future::BoxFuture, FutureExt};

#[derive(chekov::macros::EventHandler, Clone)]
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
            let _result = Account::create(&event, p).await;
            println!("Receive account opened on handler!");

            Ok(())
        }
        .boxed()
    }
}

#[chekov::event_handler]
impl chekov::event::Handler<AccountUpdated> for AccountProjector {
    fn handle(&mut self, event: &AccountUpdated) -> BoxFuture<Result<(), ()>> {
        let pool = self.pool.acquire();
        let event = event.clone();

        async move {
            if let Ok(p) = pool.await {
                if let AccountUpdated::NameChanged(account_id, _, name) = event {
                    let _result = Account::update(&account_id, &name, p).await;
                }
            }

            Ok(())
        }
        .boxed()
    }
}
