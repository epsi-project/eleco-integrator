use crate::configuration::get_configuration;
use futures::future::join_all;
use futures_lite::stream::StreamExt;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Consumer, Result,
};
use log::{error, info};
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_amqp::LapinTokioExt;

mod configuration;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // Open connection.
    let _start_connection = Instant::now();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let rabbitmq_url = format!(
        "{}://{}:{}@{}:{}/{}",
        configuration.rabbitmq.protocol,
        configuration.rabbitmq.auth.username,
        configuration.rabbitmq.auth.password.expose_secret(),
        configuration.rabbitmq.host,
        configuration.rabbitmq.port,
        configuration.rabbitmq.auth.username
    );

    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;

    info!(
        "Connected to {} in {:?}",
        configuration.rabbitmq.host,
        _start_connection.elapsed()
    );

    let channel_a = conn.create_channel().await?;
    let channel_b = conn.create_channel().await?;
    let channel_c = conn.create_channel().await?;

    let production_queue = channel_a
        .queue_declare(
            "production",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let consumption_queue = channel_b
        .queue_declare(
            "consumption",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let storage_queue = channel_c
        .queue_declare(
            "storage",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("Declared queue {}", production_queue.name());
    info!("Declared queue {}", consumption_queue.name());
    info!("Declared queue {}", storage_queue.name());

    let mut production_consumer = channel_a
        .basic_consume(
            "production",
            "production_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut consumption_consumer = channel_b
        .basic_consume(
            "consumption",
            "consumption_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut storage_consumer = channel_c
        .basic_consume(
            "storage",
            "storage_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::select! {
        _ = run_consumer(&mut production_consumer) => {
            info!("production_consumer finished");
        }
        _ = run_consumer(&mut consumption_consumer) => {
            info!("consumption_consumer finished");
        }
        _ = run_consumer(&mut storage_consumer) => {
            info!("storage_consumer finished");
        }
    }

    Ok(())
}

async fn run_consumer(consumer: &mut Consumer) {
    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumption_consumer");
        info!("Received message: {:?}", String::from_utf8(delivery.data.to_vec()).expect("Message is not a valid utf8"));
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }
}
