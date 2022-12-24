use crate::{RemoteAddr, RemoteMessage};
use actix::{Actor, Message};
use actix::dev::{MessageResponse, OneshotSender};

pub struct ResponseEnvelope(());

impl ResponseEnvelope {
    pub fn handle<F, M>(addrs: Vec<RemoteAddr>, f: F) -> ResponseEnvelope
    where
        F: Fn() -> M,
        M: RemoteMessage + Clone,
    {
        let message = f();
        for addr in addrs {
            addr.send(message.clone());
        }
        ResponseEnvelope(())
    }
}

pub trait MessageWithResponse: RemoteMessage {
    type Response: RemoteMessage;
}

impl<A, M> MessageResponse<A, M> for ResponseEnvelope 
where A: Actor, M: Message {
    fn handle(self, _ctx: &mut A::Context, _tx: Option<OneshotSender<M::Result>>) {
    }
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
            ResponseEnvelope::handle(vec![], || {
                GetResponse
            })
        }
    }

    #[actix_rt::test]
    async fn test_with_response() {
        let service = RemoteService.start();
        service.send(GetMessage);
    }
}
