use std::{pin::Pin, sync::Arc};

use anyhow::Context;
use async_std::sync::RwLock;
use espresso_types::SeqTypes;
use futures::{Stream, StreamExt as _};
use hotshot::types::Event;
use hotshot_events_service::{events, events_source::StartupInfo};
use hotshot_types::traits::node_implementation::NodeType;
use surf_disco::Client;
use tide_disco::Url;

use crate::state::GlobalState;

pub struct EventsServiceClient(Client<events::Error, <SeqTypes as NodeType>::Base>);

impl EventsServiceClient {
    pub async fn new(url: Url) -> Self {
        let client = Client::<events::Error, <SeqTypes as NodeType>::Base>::new(url.clone());

        client.connect(None).await;

        Self(client)
    }

    pub async fn get_startup_event(
        &self,
    ) -> Result<StartupInfo<SeqTypes>, hotshot_events_service::events::Error> {
        self.0.get("hotshot-events/startup_info").send().await
    }

    pub async fn get_event_stream(
        self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Result<Event<SeqTypes>, events::Error>> + Send>>>
    {
        let stream = self
            .0
            .socket("hotshot-events/events")
            .subscribe::<Event<SeqTypes>>()
            .await
            .context("failed to get event stream")?;

        Ok(stream.boxed())
    }
}

pub async fn handle_events(
    mut stream: Pin<Box<dyn Stream<Item = Result<Event<SeqTypes>, events::Error>> + Send>>,
    _state: Arc<RwLock<GlobalState>>,
) -> anyhow::Result<()> {
    while let Some(event) = stream.next().await {
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

#[cfg(any(test, feature = "testing"))]
pub mod mock {
    use std::{sync::Arc, time::Duration};

    use ::rand::{OsRng, Rng};
    use async_compatibility_layer::{art::async_spawn, logging::setup_logging};
    use async_std::{
        stream::StreamExt,
        sync::RwLock,
        task::{sleep, JoinHandle},
    };
    use espresso_types::SeqTypes;
    use hotshot::rand::{self};
    use hotshot_events_service::events_source::{EventConsumer, EventsStreamer, StartupInfo};
    use hotshot_types::{
        data::ViewNumber,
        event::{Event, EventType},
        light_client::StateKeyPair,
        signature_key::BLSPubKey,
        traits::{node_implementation::ConsensusTime, signature_key::SignatureKey},
        PeerConfig,
    };
    use portpicker::pick_unused_port;
    use surf_disco::Client;
    use tide_disco::{App, Url};
    use vbs::version::{StaticVersion, StaticVersionType};

    const NON_STAKED_NODE_COUNT: usize = 10;
    const NODE_STAKE: u64 = 1;
    const STAKED_NODES: usize = 10;
    type StaticVer01 = StaticVersion<0, 1>;

    pub fn generate_stake_table() -> Vec<PeerConfig<BLSPubKey>> {
        (0..STAKED_NODES)
            .map(|_| {
                let private_key =
                    <BLSPubKey as SignatureKey>::PrivateKey::generate(&mut rand::thread_rng());
                let pub_key = BLSPubKey::from_private(&private_key);
                let state_key_pair = StateKeyPair::generate();

                PeerConfig::<BLSPubKey> {
                    stake_table_entry: pub_key.stake_table_entry(NODE_STAKE),
                    state_ver_key: state_key_pair.ver_key(),
                }
            })
            .collect()
    }

    pub async fn generate_view_finished_events(streamer: Arc<RwLock<EventsStreamer<SeqTypes>>>) {
        sleep(Duration::from_secs(2)).await;

        let mut view_number = ViewNumber::new(1);

        let mut rng = OsRng::new().expect("Failed to obtain OsRng");
        let mut count = 1;
        loop {
            while count % 10 != 0 {
                tracing::info!("generating ViewFinished event");

                streamer
                    .write()
                    .await
                    .handle_event(Event {
                        view_number,
                        event: EventType::ViewFinished {
                            view_number: view_number - 1,
                        },
                    })
                    .await;

                view_number += 1;
                count += 1;
            }
            count = 1;

            let delay = 1000 + (u64::from(rng.next_u32()) % 1000);
            tracing::warn!("sleeping for {delay:?}ms...");

            sleep(Duration::from_millis(delay)).await
        }
    }

    pub fn run_event_service() -> (Url, JoinHandle<Result<(), std::io::Error>>, JoinHandle<()>) {
        let port = pick_unused_port().expect("no free port");
        let url = Url::parse(format!("http://localhost:{port}").as_str()).unwrap();

        let known_nodes_with_stake = generate_stake_table();

        let events_streamer = Arc::new(RwLock::new(EventsStreamer::<SeqTypes>::new(
            known_nodes_with_stake.clone(),
            NON_STAKED_NODE_COUNT,
        )));

        let mut app =
            App::<_, hotshot_events_service::events::Error>::with_state(events_streamer.clone());

        let hotshot_events_api =
            hotshot_events_service::events::define_api::<_, _, StaticVer01>(&Default::default())
                .expect("Failed to define hotshot eventsAPI");

        app.register_module("api", hotshot_events_api)
            .expect("Failed to register hotshot events API");

        let events_api_handle = async_spawn(app.serve(url.clone(), StaticVer01::instance()));
        let generate_events_handle = async_spawn(generate_view_finished_events(events_streamer));

        let api_url = url.join("api").unwrap();

        (api_url, events_api_handle, generate_events_handle)
    }

    #[async_std::test]
    async fn test_mock_events_service() {
        setup_logging();
        let (url, _handle1, _handle2) = run_event_service();

        tracing::info!("running event service");

        let client = Client::<hotshot_events_service::events::Error, StaticVer01>::new(url);
        client.connect(None).await;

        let startup_info: StartupInfo<SeqTypes> = client
            .get("startup_info")
            .send()
            .await
            .expect("failed to get startup_info");

        tracing::info!("startup info {startup_info:?}");

        let mut events = client
            .socket("events")
            .subscribe::<Event<SeqTypes>>()
            .await
            .unwrap();

        let mut num_of_events_received = 0;
        while let Some(event) = events.next().await {
            let event = event.unwrap();
            tracing::info!("event {event:?}");

            num_of_events_received += 1;

            if num_of_events_received == 15 {
                break;
            }
        }
    }
}
