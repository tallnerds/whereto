use futures::future::{BoxFuture, FutureExt};
use futures::stream::{self, StreamExt};
use reqwest::{Client, Url};
use std::collections::HashMap;
use std::sync::Arc;

const POOL_SIZE: usize = 5;

pub struct Processor {
    pub hosts: Vec<Url>,
    client: Client,
}

impl Processor {
    pub fn from_hosts(hosts: Vec<Url>) -> Self {
        Processor {
            hosts,
            client: Client::new(),
        }
    }

    pub async fn process(&self) -> Result<HashMap<Url, Url>, reqwest::Error> {
        let client = Arc::new(self.client.clone());

        let results = stream::iter(self.hosts.clone())
            .map(move |h| {
                let client = client.clone();

                // TODO: Fix this unwrap, probably need a custom error
                async move { resolve_url(client, h).await.unwrap() }
            })
            .buffered(POOL_SIZE)
            .collect::<Vec<Url>>()
            .await;

        let hashmap: HashMap<Url, Url> = self
            .hosts
            .iter()
            .zip(results)
            .filter(|(src, dest)| src != &dest)
            .fold(HashMap::new(), |mut map, (o, r)| {
                map.insert(o.to_owned(), r);
                map
            });

        Ok(hashmap)
    }
}

fn resolve_url(client: Arc<Client>, url: Url) -> BoxFuture<'static, Result<Url, reqwest::Error>> {
    async move {
        let response = client.get(url.clone()).send().await?;

        Ok(response.url().to_owned())
    }
    .boxed()
}
