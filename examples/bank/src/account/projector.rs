use super::*;
use crate::DefaultApp;
use actix::AsyncContext;
use actix::Context;
use actix::SystemService;
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
        let pool = self.pool.clone();
        Box::pin(async move {
            let _result = Account::create(&event, &pool).await;
            println!("Receive account opened on handler!");

            Ok(())
        })
    }
}

impl chekov::event::Handler<AccountUpdated> for AccountProjector {
    fn handle(
        &mut self,
        _event: &AccountUpdated,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> {
        // let _result = Account::create(event, &self.pool).await;
        Box::pin(async move { Ok(()) })
    }
}
