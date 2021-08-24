use super::*;
use chekov::event::EventHandlerInstance;
use futures::Future;

pub struct AccountProjector {
    pub pool: PgPool,
}

impl EventHandler for AccountProjector {
    fn started<A: Application>(&mut self, ctx: &mut actix::Context<EventHandlerInstance<A, Self>>) {
        self.listen::<_, AccountOpened>(ctx);
        self.listen::<_, AccountUpdated>(ctx);
    }
}

impl chekov::event::Handler<AccountOpened> for AccountProjector {
    fn handle(
        &mut self,
        event: &AccountOpened,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> {
        let event = event.clone();
        let pool = self.pool.acquire();
        Box::pin(async move {
            let p = pool.await.unwrap();
            let _result = Account::create(&event, p).await;
            println!("Receive account opened on handler!");

            Ok(())
        })
    }
}

impl chekov::event::Handler<AccountUpdated> for AccountProjector {
    fn handle(
        &mut self,
        event: &AccountUpdated,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> {
        let pool = self.pool.acquire();
        let event = event.clone();
        Box::pin(async move {
            if let Ok(p) = pool.await {
                let _result = Account::update(&event, p).await;
            }

            Ok(())
        })
    }
}
