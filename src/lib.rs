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
      msgs.put("test", "test").unwrap().execute().await.unwrap();
      Response::from_html(hex::encode(&writer))
    })
    .run(req, env)
    .await
}
