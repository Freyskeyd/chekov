use crate::aggregate::instance::internal::CommandExecutionResult;
use crate::command::{Command, Handler};
use crate::error::CommandExecutorError;
use crate::message::Dispatch;
use crate::message::{AggregateVersion, ResolveAndApply, ResolveAndApplyMany};
use crate::{Aggregate, Application};
use actix::prelude::*;
use actix::Handler as ActixHandler;
use event_store::prelude::SubscriptionNotification;
use tracing::trace;

use super::AggregateInstance;

impl<A: Aggregate> ActixHandler<AggregateVersion> for AggregateInstance<A> {
    type Result = i64;

    fn handle(&mut self, _: AggregateVersion, _: &mut Self::Context) -> Self::Result {
        self.current_version
    }
}

impl<A: Aggregate> Actor for AggregateInstance<A> {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("Aggregate {:?} started", std::any::type_name::<Self>());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("Aggregate {:?} stopped", std::any::type_name::<Self>());
    }
}

impl<C: Command, A: Application> ActixHandler<Dispatch<C, A>> for AggregateInstance<C::Executor>
where
    C::CommandHandler: Handler<C, C::Executor>,
{
    type Result = ResponseActFuture<Self, Result<Vec<C::Event>, CommandExecutorError>>;

    #[tracing::instrument(
        name = "AggregateInstance",
        skip(self, _ctx, cmd),
        fields(correlation_id = %cmd.metadatas.correlation_id, aggregate_id = %cmd.command.identifier(), aggregate_type = %::std::any::type_name::<C::Executor>())
    )]
    fn handle(&mut self, cmd: Dispatch<C, A>, _ctx: &mut Self::Context) -> Self::Result {
        trace!("Executing command {}", std::any::type_name::<C>(),);

        let mutable_state = self.create_mutable_state();
        let current_version = self.current_version;

        // TODO: This code block isn't safe in an aggregate context
        //
        // Returning a boxed future, will not prevent the state to be altered or other event to be
        // proceed. A workaround to make the instance of this aggregate sequential would be to
        // return a future that wait for the result of the execute_and_apply. The future of the
        // execute_and_apply need to be dispatch to the Context::wait to lock any further message
        // to be proceed.
        Box::pin(
            async move { Self::execute_and_apply(mutable_state, cmd, current_version).await }
                .into_actor(self)
                .map(|result, actor, _| match result {
                    Ok(CommandExecutionResult {
                        events,
                        new_version,
                        state,
                    }) => {
                        actor.current_version = new_version;
                        actor.inner = state;
                        Ok(events)
                    }
                    Err(e) => Err(e),
                }),
        )
    }
}

impl<A: Aggregate> ActixHandler<ResolveAndApply> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApply, _: &mut Self::Context) -> Self::Result {
        self.apply_recorded_event(&msg.0).map_err(|_| ())
    }
}

impl<A: Aggregate> ActixHandler<ResolveAndApplyMany> for AggregateInstance<A> {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: ResolveAndApplyMany, _: &mut Self::Context) -> Self::Result {
        for event in msg.0 {
            let _ = self.apply_recorded_event(&event);
        }

        Ok(())
    }
}

impl<A: Aggregate> ActixHandler<SubscriptionNotification> for AggregateInstance<A> {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: SubscriptionNotification, _: &mut Self::Context) -> Self::Result {
        match msg {
            SubscriptionNotification::Events(events) => {
                for event in events.as_ref() {
                    let _ = self.apply_recorded_event(&event);
                }
            }
            SubscriptionNotification::OwnedEvents(events) => {
                for event in events.iter() {
                    let _ = self.apply_recorded_event(event);
                }
            }
            SubscriptionNotification::Subscribed => {}
        }

        Box::pin(async { Ok(()) }.into_actor(self))
    }
}
