use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
  // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
  // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
  // routes to access and share using the `ctx.data()` method.
  let router = Router::new();

  #[derive(Deserialize, Serialize)]
  struct Account {
    id: u64,
  }

  router
    .get_async("/test", |_req, ctx| async move {
      let mut writer = Vec::new();
      let value = Account { id: 3 };
      ciborium::ser::into_writer(&value, &mut writer).unwrap();
      let msgs = ctx.kv("KVTEST")?;
      msgs.put_bytes("test", &writer).unwrap().execute().await.unwrap();
      Response::from_html(hex::encode(&writer))
    })
    .get("/", |_, ctx| {
      // Accept / handle a websocket connection
      let pair = WebSocketPair::new()?;
      let server = pair.server;
      server.accept()?;

      let some_namespace_kv = ctx.kv("KVTEST")?;

      wasm_bindgen_futures::spawn_local(async move {
        let mut event_stream = server.events().expect("could not open stream");

        while let Some(event) = event_stream.next().await {
          match event.expect("received error in websocket") {
            WebsocketEvent::Message(msg) => {
              if let Some(text) = msg.text() {
                server.send_with_str(text).expect("could not relay text");
              }
            }
            WebsocketEvent::Close(_) => {
              // Sets a key in a test KV so the integration tests can query if we
              // actually got the close event. We can't use the shared dat a for this
              // because miniflare resets that every request.
              some_namespace_kv
                .put("got-close-event", "true")
                .unwrap()
                .execute()
                .await
                .unwrap();
            }
          }
        }
      });

      Response::from_websocket(pair.client)
    })
    .run(req, env)
    .await
}
