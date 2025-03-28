use serde_json::Value;

#[derive(Clone)]
struct State {
    client: reqwest::Client,
}

#[derive(Clone)]
struct SharedClient {
    actor: cooper::Actor<State>,
}

impl SharedClient {
    pub fn new() -> Self {
        Self {
            actor: cooper::Actor::new(State {
                client: reqwest::Client::new(),
            }),
        }
    }

    pub async fn fetch(&self, url: impl Into<String>) -> Value {
        let url = url.into();
        self.actor
            .call(|_, state| {
                Box::pin(async move {
                    let res = state.client.get(url).send().await.unwrap();
                    Some(res.json().await.unwrap())
                })
            })
            .await
    }
}

#[cfg(not(feature = "tokio"))]
#[tokio::main]
async fn main() {
    println!(
        "This example fails to run with default features, 
as it uses `reqwest` internally which in turn
only runs on a tokio runtime."
    );
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
    let client = SharedClient::new();
    let res = client.fetch("https://httpbin.org/json").await;
    dbg!(res);
}
