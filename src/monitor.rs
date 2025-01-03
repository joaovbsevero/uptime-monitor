use chrono::Utc;
use futures::stream::StreamExt;
use mongodb::Database;
use mongodb::{bson::doc, Collection};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::models::{Check, CheckHistory, Frequency, HTTPMethod, Status, WebhookData};

pub(crate) async fn start(db: Database) {
    let checks_collection = db.collection::<Check>("checks");
    let history_collection = db.collection::<CheckHistory>("checks_history");

    let client = Client::new();
    info!("Starting monitor task");
    loop {
        fetch_and_execute_checks(&checks_collection, &history_collection, &client).await;
        tokio::time::sleep(Duration::from_secs(3600)).await; // Sleep for 1 hour
    }
}

async fn fetch_and_execute_checks(
    checks_collection: &Collection<Check>,
    history_collection: &Collection<CheckHistory>,
    client: &Client,
) {
    info!("Fetching checks from database");
    let cursor = checks_collection.find(doc! {}).await;
    if cursor.is_err() {
        error!("Error fetching checks from database");
        return;
    }
    let mut cursor = cursor.unwrap();

    while let Some(result) = cursor.next().await {
        if let Ok(check) = result {
            let history = history_collection
                .find_one(doc! {
                    "check_id": check._id,
                })
                .await;

            if history.is_err() {
                warn!("Error fetching history for check '{}'", check._id);
                continue;
            }
            let history = history.unwrap();

            match (&check.frequency, history) {
                (Frequency::Hourly, Some(h)) => {
                    if h.created_at > Utc::now() - chrono::Duration::weeks(1) {
                        // If previous ping was greater then one week ago, ignore it
                        info!(
                            "Skipping check '{}' because it was executed less than one hour ago",
                            check._id
                        );
                        continue;
                    }
                }
                (Frequency::Daily, Some(h)) => {
                    if h.created_at > Utc::now() - chrono::Duration::days(1) {
                        // If previous ping was greater then one day ago, ignore it
                        info!(
                            "Skipping check '{}' because it was executed less than one day ago",
                            check._id
                        );
                        continue;
                    }
                }
                (Frequency::Weekly, Some(h)) => {
                    if h.created_at > Utc::now() - chrono::Duration::weeks(1) {
                        // If previous ping was greater then one week ago, ignore it
                        info!(
                            "Skipping check '{}' because it was executed less than one week ago",
                            check._id
                        );
                        continue;
                    }
                }
                _ => (),
            };

            let details = execute_check(&check, client).await;

            let status = details
                .as_ref()
                .map_or_else(|_| Status::Error, |_| Status::Ok);
            let details = details.err();

            let check_history = CheckHistory::new(check._id, status.clone(), details.clone());
            let previous_status = check_history.status.clone();

            let result = history_collection.insert_one(check_history).await;
            if result.is_err() {
                warn!("Error saving history for check '{}'", check._id);
            }

            let data = WebhookData::new(status, details, check);
            if let Some(ref hook) = data.check.hook {
                if data.status == Status::Error || previous_status == Status::Error {
                    let result = client.post(hook).json(&data).send().await;
                    if result.is_err() {
                        warn!("Error sending hook for check '{}'", data.check._id);
                    }
                } else {
                    info!(
                        "Skipping hook for check '{}', because status is OK and previous status was OK as well",
                        data.check._id
                    );
                }
            }
        }
    }
    info!("Finished executing checks");
}

async fn execute_check(check: &Check, client: &Client) -> Result<(), String> {
    match check.method {
        HTTPMethod::GET => {
            let response = client.get(&check.url).send().await;
            match response {
                Ok(resp) => {
                    if resp.status().as_u16() > 399 {
                        return Err(format!(
                            "Endpoint returned error status code: '{}'",
                            resp.status()
                        ));
                    }

                    if let Some(ref expected_body) = check.expected_body {
                        match resp.json::<Value>().await {
                            Ok(v) => {
                                if expected_body == &v {
                                    Ok(())
                                } else {
                                    Err(format!(
                                        "Endpoint returned unexpected body: {} != {}",
                                        v, expected_body
                                    ))
                                }
                            }
                            Err(err) => Err(format!(
                                "Malformed request: GET '{}', error '{}'",
                                check.url,
                                err.to_string()
                            )),
                        }
                    } else {
                        Ok(())
                    }
                }
                Err(err) => Err(err.to_string()),
            }
        }
        HTTPMethod::HEAD => {
            let response = client.head(&check.url).send().await;
            match response {
                Ok(resp) => {
                    if resp.status().as_u16() > 399 {
                        Err(format!(
                            "Endpoint returned error status code: '{}'",
                            resp.status()
                        ))
                    } else {
                        Ok(())
                    }
                }
                Err(err) => Err(format!(
                    "Malformed request: HEAD '{}', error '{}'",
                    check.url,
                    err.to_string()
                )),
            }
        }
    }
}
