use super::*;
use chekov::event_handler::EventHandlerInstance;

pub struct AccountProjector {
    pool: PgPool,
}

impl EventHandler for AccountProjector {
    fn started<A: Application>(&mut self, ctx: &mut actix::Context<EventHandlerInstance<A, Self>>) {
        self.listen::<_, AccountOpened>(ctx);
        self.listen::<_, AccountUpdated>(ctx);
    }
}

impl chekov::event_handler::Listening for AccountProjector {}

#[async_trait::async_trait]
impl chekov::event::Handler<AccountOpened> for AccountProjector {
    async fn handle(&self, event: &AccountOpened) -> Result<(), ()> {
        let _result = Account::create(event, &self.pool).await;
        Ok(())
    }
}
