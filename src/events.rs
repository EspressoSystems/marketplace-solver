use async_std::stream::StreamExt;
use hotshot_events_service::{events, events_source::BuilderEvent};
use hotshot_types::traits::node_implementation::NodeType;

use surf_disco::{
    socket::{Connection, Unsupported},
    Client,
};
use tide_disco::Url;

pub struct EventHandler;

impl EventHandler {
    pub async fn run<TYPES: NodeType>(url: Url) -> anyhow::Result<()> {
        let mut events = Self::connect_service::<TYPES>(url).await.unwrap();

        while let Some(event) = events.next().await {
            let event = event?;

            tracing::info!("received event {event:?}");
        }

        Ok(())
    }

    pub async fn connect_service<TYPES: NodeType>(
        url: Url,
    ) -> Result<
        Connection<BuilderEvent<TYPES>, Unsupported, events::Error, <TYPES as NodeType>::Base>,
        events::Error,
    > {
        let client = Client::<events::Error, TYPES::Base>::new(url.clone());

        client.connect(None).await;

        client
            .socket("hotshot-events/events")
            .subscribe::<BuilderEvent<TYPES>>()
            .await
    }
}
