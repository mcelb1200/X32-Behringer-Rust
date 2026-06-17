use crate::client::MixerClient;
use crate::transport::MixerTransport;
use async_trait::async_trait;
use osc_lib::OscMessage;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;

pub struct MockTransport {
    pub sent_messages: Mutex<Vec<OscMessage>>,
    pub receive_queue: Mutex<mpsc::Receiver<OscMessage>>,
}

impl MockTransport {
    pub fn new() -> (Arc<Self>, mpsc::Sender<OscMessage>) {
        let (tx, rx) = mpsc::channel(10);
        let transport = Arc::new(Self {
            sent_messages: Mutex::new(Vec::new()),
            receive_queue: Mutex::new(rx),
        });
        (transport, tx)
    }

    pub async fn get_sent_messages(&self) -> Vec<OscMessage> {
        self.sent_messages.lock().await.clone()
    }
}

#[async_trait]
impl MixerTransport for MockTransport {
    async fn send(&self, msg: OscMessage) -> crate::error::Result<()> {
        self.sent_messages.lock().await.push(msg);
        Ok(())
    }

    async fn recv(&self) -> crate::error::Result<OscMessage> {
        let mut rx = self.receive_queue.lock().await;
        match rx.recv().await {
            Some(msg) => Ok(msg),
            None => {
                // If channel is closed, block forever so the background receiver task doesn't loop tightly
                let () = std::future::pending().await;
                unreachable!()
            }
        }
    }
}

#[tokio::test]
async fn test_mixer_client_new_heartbeat() {
    let (transport, _tx) = MockTransport::new();
    let _client = MixerClient::new(transport.clone(), true);

    // Initial tick happens immediately, sending /xremote
    tokio::time::sleep(Duration::from_millis(50)).await;

    let sent = transport.get_sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/xremote");
    assert!(sent[0].args.is_empty());

    // Stop heartbeat to clean up the task and prevent lingering processes.
    _client.stop_heartbeat();
}

#[tokio::test]
async fn test_mixer_client_send_message() {
    let (transport, _tx) = MockTransport::new();
    let client = MixerClient::new(transport.clone(), false);

    client
        .send_message("/ch/01/mix/fader", vec![osc_lib::OscArg::Float(0.75)])
        .await
        .unwrap();

    let sent = transport.get_sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/ch/01/mix/fader");
    assert_eq!(sent[0].args.len(), 1);
    match &sent[0].args[0] {
        osc_lib::OscArg::Float(f) => assert_eq!(*f, 0.75),
        _ => panic!("Expected Float argument"),
    }
}

#[tokio::test]
async fn test_mixer_client_query_value() {
    let (transport, tx) = MockTransport::new();
    let client = MixerClient::new(transport.clone(), false);

    // Spawn a task to send the response back shortly after
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let response = OscMessage::new(
            "/ch/01/mix/fader".to_string(),
            vec![osc_lib::OscArg::Float(0.5)],
        );
        tx_clone.send(response).await.unwrap();
    });

    let res = client.query_value("/ch/01/mix/fader").await.unwrap();

    // Verify it sent a query
    let sent = transport.get_sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/ch/01/mix/fader");
    assert!(sent[0].args.is_empty());

    // Verify it parsed the result
    match res {
        osc_lib::OscArg::Float(f) => assert_eq!(f, 0.5),
        _ => panic!("Expected Float argument"),
    }
}

#[tokio::test]
async fn test_mixer_client_query_node() {
    let (transport, tx) = MockTransport::new();
    let client = MixerClient::new(transport.clone(), false);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        // The mixer responds with `/node` followed by a single string of parameters
        let response = OscMessage::new(
            "node".to_string(),
            vec![osc_lib::OscArg::String("ch/01 ON 100".to_string())],
        );
        tx_clone.send(response).await.unwrap();
    });

    let res = client.query_node("ch/01").await.unwrap();

    // Verify it sent a node query
    let sent = transport.get_sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/node");
    assert_eq!(sent[0].args.len(), 1);
    match &sent[0].args[0] {
        osc_lib::OscArg::String(s) => assert_eq!(s, "ch/01"),
        _ => panic!("Expected String argument"),
    }

    // Verify parsed result
    assert_eq!(res, "ch/01 ON 100");
}

#[tokio::test]
async fn test_mixer_client_probe_success() {
    let (transport, tx) = MockTransport::new();
    let client = MixerClient::new(transport.clone(), false);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let response = OscMessage::new("/info".to_string(), vec![]);
        tx_clone.send(response).await.unwrap();
    });

    let res = client.probe().await;

    // Verify it sent an info query
    let sent = transport.get_sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].path, "/info");

    // Verify probe result
    assert!(res);
}

#[tokio::test]
async fn test_mixer_client_probe_timeout() {
    let (transport, _tx) = MockTransport::new();
    let client = MixerClient::new(transport.clone(), false);

    // No response sent

    let res = client.probe().await;

    // Verify probe result
    assert!(!res);
}
