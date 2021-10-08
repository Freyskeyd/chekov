use actix::prelude::*;

use super::SubscriptionNotification;

#[derive(Debug)]
pub struct Subscriber {
    pub recipient: Recipient<SubscriptionNotification>,
}

impl Actor for Subscriber {
    type Context = Context<Self>;
}

impl Handler<SubscriptionNotification> for Subscriber {
    type Result = ResponseActFuture<Self, <SubscriptionNotification as Message>::Result>;

    fn handle(&mut self, msg: SubscriptionNotification, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            self.recipient
                .send(msg)
                .into_actor(self)
                .map(|res, _, _| match res {
                    Ok(r) => r,
                    Err(_) => Err(()),
                }),
        )
    }
}
