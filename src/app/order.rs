use crate::util::{get_market_quote, publish_order, send_dm};

use anyhow::Result;
use dotenvy::var;
use mostro_core::{Action, Message};
use nostr_sdk::prelude::ToBech32;
use nostr_sdk::{Client, Event, Keys};
use sqlx::{Pool, Sqlite};

pub async fn order_action(
    msg: Message,
    event: &Event,
    my_keys: &Keys,
    client: &Client,
    pool: &Pool<Sqlite>,
) -> Result<()> {
    if let Some(order) = msg.get_order() {
        let quote = get_market_quote(&order.fiat_amount, &order.fiat_code, &0).await?;
        if quote > var("MAX_ORDER_AMOUNT").unwrap().parse::<i64>().unwrap() {
            let message = Message::new(0, order.id, None, Action::CantDo, None);
            let message = message.as_json()?;
            send_dm(client, my_keys, &event.pubkey, message).await?;

            return Ok(());
        }

        let initiator_ephemeral_pubkey = event.pubkey.to_bech32()?;
        let master_pubkey = match msg.pubkey {
            Some(ref pk) => pk,
            None => {
                // We create a Message
                let message = Message::new(0, order.id, None, Action::CantDo, None);
                let message = message.as_json()?;
                send_dm(client, my_keys, &event.pubkey, message).await?;

                return Ok(());
            }
        };

        publish_order(
            pool,
            client,
            my_keys,
            order,
            &initiator_ephemeral_pubkey,
            master_pubkey,
            event.pubkey,
        )
        .await?;
    }
    Ok(())
}
