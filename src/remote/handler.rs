use crate::{RemoteAddr, RemoteMessage};
use actix::dev::{MessageResponse, OneshotSender};
use actix::{Actor, Message};
use std::convert::TryInto;

pub struct ResponseEnvelope(());

impl ResponseEnvelope {
    pub fn handle<F, M, I>(addrs: Vec<RemoteAddr>, f: F) -> ResponseEnvelope
    where
        F: Fn() -> I,
        M: RemoteMessage + Clone,
        I: Into<M>,
    {
        let message = f().into();
        for addr in addrs {
            addr.send(message.clone());
        }
        ResponseEnvelope(())
    }
    pub fn try_handle<F, M, T, D>(addrs: Vec<RemoteAddr>, default: D, f: F) -> ResponseEnvelope
    where
        F: Fn() -> T,
        D: RemoteMessage + Clone,
        M: RemoteMessage + Clone,
        T: TryInto<M>,
        <T as TryInto<M>>::Error: std::fmt::Debug,
    {
        match f().try_into() {
            Ok(msg) => {
                for addr in addrs {
                    addr.send(msg.clone());
                }
            }
            Err(err) => {
                log::error!("Error occured:\n{err:?}");
                for addr in addrs {
                    addr.send(default.clone());
                }
            }
        }
        ResponseEnvelope(())
    }
}

pub trait MessageWithResponse: RemoteMessage {
    type Response: RemoteMessage;
}

impl<A, M> MessageResponse<A, M> for ResponseEnvelope
where
    A: Actor,
    M: Message,
{
    fn handle(self, _ctx: &mut A::Context, _tx: Option<OneshotSender<M::Result>>) {}
}

#[cfg(test)]
mod tests {
    use super::MessageResponse;
    use crate::prelude::*;
    use crate::remote::handler::ResponseEnvelope;
    use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
    use serde::{Deserialize, Serialize};

    #[derive(RemoteMessage, Serialize, Deserialize)]
    #[with_response]
    pub struct GetMessage;

    #[derive(RemoteMessage, Serialize, Deserialize, Clone)]
    pub struct GetResponse;

    #[derive(RemoteActor)]
    #[remote_messages(GetMessage)]
    pub struct RemoteService;

    impl Actor for RemoteService {
        type Context = Context<Self>;
    }

    impl Handler<GetMessage> for RemoteService {
        type Result = ResponseEnvelope;

        fn handle(&mut self, _msg: GetMessage, _: &mut Self::Context) -> ResponseEnvelope {
            ResponseEnvelope::handle(vec![], || GetResponse)
        }
    }

    #[actix_rt::test]
    async fn test_with_response() {
        let service = RemoteService.start();
        service.send(GetMessage);
    }
}
