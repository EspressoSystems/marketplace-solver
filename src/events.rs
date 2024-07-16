use async_std::stream::StreamExt;
use hotshot::traits::election::static_committee::GeneralStaticCommittee;
use hotshot_events_service::{
    events,
    events_source::{BuilderEvent, BuilderEventType},
};
use hotshot_types::traits::{election::Membership, node_implementation::NodeType};

use surf_disco::{
    socket::{Connection, Unsupported},
    Client,
};
use tide_disco::Url;

pub struct EventHandler;

impl EventHandler {
    pub async fn run<TYPES: NodeType>(url: Url) -> anyhow::Result<()> {
        let mut events = Self::connect_service::<TYPES>(url).await?;

        while let Some(event) = events.next().await {
            let event = event?;

            tracing::info!("received event {:?}", event.event);

            // TODO ED: Remove this lint later
            #[allow(clippy::single_match)]
            match event.event {
                BuilderEventType::StartupInfo {
                    known_node_with_stake,
                    non_staked_node_count,
                } => {
                    // >>> use membership?
                    let _membership: GeneralStaticCommittee<
                        TYPES,
                        <TYPES as NodeType>::SignatureKey,
                    > = GeneralStaticCommittee::create_election(
                        known_node_with_stake.clone(),
                        known_node_with_stake.clone(),
                        0,
                    );

                    tracing::info!(
                        "Startup info: Known nodes with stake: {:?}, Non-staked node count: {:?}",
                        known_node_with_stake,
                        non_staked_node_count
                    );
                }
                // todo: handle events
                _ => (),
            }
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
