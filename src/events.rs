use std::sync::Arc;

use async_std::{stream::StreamExt, sync::RwLock};
use hotshot::types::Event;
use hotshot_events_service::events;
use hotshot_types::traits::node_implementation::NodeType;
use surf_disco::{
    socket::{Connection, Unsupported},
    Client,
};
use tide_disco::Url;

use crate::state::SolverState;

pub struct EventHandler;

impl EventHandler {
    pub async fn run<TYPES: NodeType>(
        url: Url,
        _state: Arc<RwLock<SolverState>>,
    ) -> anyhow::Result<()> {
        let mut events = Self::connect_service::<TYPES>(url).await?;

        while let Some(event) = events.next().await {
            let event = event?;

            tracing::info!("received event {:?}", event.event);

            // TODO ED: Remove this lint later
            #[allow(clippy::single_match)]
            match event.event {
                hotshot::types::EventType::ViewFinished { view_number } => {
                    tracing::info!("received view finished event {view_number:?}")
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub async fn connect_service<TYPES: NodeType>(
        url: Url,
    ) -> Result<
        Connection<Event<TYPES>, Unsupported, events::Error, <TYPES as NodeType>::Base>,
        events::Error,
    > {
        let client = Client::<events::Error, TYPES::Base>::new(url.clone());

        client.connect(None).await;

        client
            .socket("hotshot-events/events")
            .subscribe::<Event<TYPES>>()
            .await
    }
}
