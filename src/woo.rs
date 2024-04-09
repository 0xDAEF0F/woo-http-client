use crate::constants::{PROXY_IP, PROXY_URL, WOO_API_BASE_URL, WOO_API_BASE_URL_STAGING};
use crate::woo_data_structs::{CancelOrder, CancelOrderRes, SendOrderRes, WooOrder};
use anyhow::Ok;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use reqwest::header::{self, HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use url::Url;

enum Environment {
    Production,
    Staging,
}

struct Woo {
    http_client: reqwest::Client,
    base_url: Url,
    api_secret: String,
}

impl Woo {
    fn new(environment: Environment) -> Self {
        dotenv().ok();

        let (base_url, api_key, api_secret) = match environment {
            Environment::Production => (
                Url::parse(WOO_API_BASE_URL).unwrap(),
                dotenv::var("WOO_API_KEY").expect("woo api key missing in .env"),
                dotenv::var("WOO_API_SECRET").expect("woo api secret missing in .env"),
            ),
            Environment::Staging => (
                Url::parse(WOO_API_BASE_URL_STAGING).unwrap(),
                dotenv::var("WOO_API_KEY_STAGING").expect("woo api staging key missing in .env"),
                dotenv::var("WOO_API_SECRET_STAGING")
                    .expect("woo api staging secret missing in .env"),
            ),
        };

        let proxy_url: Url = Url::parse(PROXY_URL).unwrap();

        let proxy_username = dotenv::var("PROXY_USERNAME").expect("proxy username missing in .env");
        let proxy_password = dotenv::var("PROXY_PASSWORD").expect("proxy password missing in .env");

        let proxy = reqwest::Proxy::all(proxy_url)
            .expect("failed to create proxy")
            .basic_auth(&proxy_username, &proxy_password);

        let mut default_headers = header::HeaderMap::new();
        default_headers.insert("x-api-key", api_key.parse().unwrap());

        let http_client = reqwest::Client::builder()
            .proxy(proxy)
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self {
            http_client,
            base_url,
            api_secret,
        }
    }

    async fn create_order(&mut self, order: WooOrder) -> anyhow::Result<SendOrderRes> {
        self.base_url.set_path("v1/order");

        let timestamp = chrono::Utc::now().timestamp_millis();

        // this part is to handle the alphabetical order of the query string
        // `url_encoded` is just an intermediate step
        let url_encoded = serde_qs::to_string(&order)?;
        let deserialized: BTreeMap<String, String> = serde_qs::from_str(&url_encoded)?;

        let req_builder = self
            .http_client
            .post(self.base_url.clone())
            .header("x-api-timestamp", timestamp)
            .header(
                "x-api-signature",
                Woo::generate_hmac_sha256_signature(
                    Woo::generate_sorted_query_string(&order),
                    timestamp as u64,
                    self.api_secret.clone(),
                ),
            )
            .form(&deserialized);

        Ok(req_builder.send().await?.json().await?)
    }

    async fn cancel_order(&mut self, cancel_order: CancelOrder) -> anyhow::Result<CancelOrderRes> {
        self.base_url.set_path("v1/order");

        let timestamp = chrono::Utc::now().timestamp_millis();

        // this part is to handle the alphabetical order of the query string
        // `url_encoded` is just an intermediate step
        let url_encoded = serde_qs::to_string(&cancel_order)?;
        let deserialized: BTreeMap<String, String> = serde_qs::from_str(&url_encoded)?;

        let req_builder = self
            .http_client
            .delete(self.base_url.clone())
            .header("x-api-timestamp", timestamp)
            .header(
                "x-api-signature",
                Woo::generate_hmac_sha256_signature(
                    Woo::generate_sorted_query_string(&cancel_order),
                    timestamp as u64,
                    self.api_secret.clone(),
                ),
            )
            .form(&deserialized);

        Ok(req_builder.send().await?.json().await?)
    }

    fn generate_sorted_query_string<P>(body: P) -> String
    where
        P: Serialize,
    {
        let unsorted_query_string =
            serde_qs::to_string(&body).expect("fail to serialize to query string");

        let mut sorted_query_string = unsorted_query_string.split('&').collect::<Vec<&str>>();
        sorted_query_string.sort();

        sorted_query_string.join("&")
    }

    fn generate_hmac_sha256_signature(
        sorted_query_string: String,
        timestamp: u64,
        secret_key: String,
    ) -> String {
        let concatted = format!("{}|{}", sorted_query_string, timestamp);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).expect("HMAC failed");
        mac.update(concatted.as_bytes());

        hex::encode(mac.finalize().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_woo_system_status() {
        let mut url = Url::parse(WOO_API_BASE_URL).unwrap();
        url.set_path("v1/public/system_info");

        let body = reqwest::get(url).await.expect("failed request");

        let status = body.status();
        assert_eq!(status.as_u16(), 200);

        #[derive(Debug, Deserialize)]
        struct WooSystemStatus {
            success: bool,
            data: Data,
            timestamp: u64,
        }

        #[derive(Debug, Deserialize)]
        struct Data {
            status: u8,
            msg: String,
        }

        let body: WooSystemStatus = body.json().await.expect("failed to parse json");
        assert!(body.success);
    }

    #[tokio::test]
    async fn get_woo_history() {
        dotenv().ok();

        let mut woo_url: Url = Url::parse(WOO_API_BASE_URL).unwrap();
        woo_url.set_path("v1/client/trades");

        let woo_api_key = dotenv::var("WOO_API_KEY").unwrap();
        let woo_api_secret = dotenv::var("WOO_API_SECRET").unwrap();

        let proxy_url: Url = Url::parse(PROXY_URL).unwrap();

        let proxy_username = dotenv::var("PROXY_USERNAME").unwrap();
        let proxy_password = dotenv::var("PROXY_PASSWORD").unwrap();

        let timestamp = chrono::Utc::now().timestamp_millis();

        let proxy = reqwest::Proxy::all(proxy_url.clone())
            .unwrap()
            .basic_auth(&proxy_username, &proxy_password);

        let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();

        let request = http_client
            .get(woo_url.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("x-api-key", woo_api_key)
            .header("x-api-timestamp", timestamp)
            .header(
                "x-api-signature",
                Woo::generate_hmac_sha256_signature(
                    "".to_string(),
                    timestamp as u64,
                    woo_api_secret.to_string(),
                ),
            );

        let response = request.send().await.expect("failed to send request");

        let status = response.status();

        assert_eq!(status.as_u16(), 200);
    }

    #[tokio::test]
    async fn send_order() {
        let mut woo = Woo::new(super::Environment::Staging);

        let order = WooOrder {
            order_price: Some(1.0),
            order_quantity: Some(2.0),
            order_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            symbol: "SPOT_ULP_USDT".to_string(),
            client_order_id: None,
            order_tag: None,
            order_amount: None,
            reduce_only: None,
            visible_quantity: None,
            position_side: None,
        };

        let order_created = woo.create_order(order).await.unwrap();

        assert!(order_created.success);
    }

    #[tokio::test]
    async fn cancel_order() {
        let mut woo = Woo::new(super::Environment::Staging);

        let order = WooOrder {
            order_price: Some(1.0),
            order_quantity: Some(2.0),
            order_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            symbol: "SPOT_ULP_USDT".to_string(),
            client_order_id: None,
            order_tag: None,
            order_amount: None,
            reduce_only: None,
            visible_quantity: None,
            position_side: None,
        };

        let order_created = woo.create_order(order).await.unwrap();

        assert!(order_created.success);

        let cancel_order = CancelOrder {
            order_id: order_created.order_id,
            symbol: "SPOT_ULP_USDT".to_string(),
        };

        let order_cancelled = woo.cancel_order(cancel_order).await.unwrap();

        assert!(order_cancelled.success);
    }

    #[test]
    fn test_hash_order() {
        let order = WooOrder {
            order_price: Some(9000.0),
            order_quantity: Some(0.11),
            order_type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            symbol: "SPOT_BTC_USDT".to_string(),
            client_order_id: None,
            order_tag: None,
            order_amount: None,
            reduce_only: None,
            visible_quantity: None,
            position_side: None,
        };

        let sorted_query_string = Woo::generate_sorted_query_string(&order);

        let signature = Woo::generate_hmac_sha256_signature(
            sorted_query_string,
            1578565539808,
            "QHKRXHPAW1MC9YGZMAT8YDJG2HPR".to_string(),
        );

        assert_eq!(
            signature,
            "20da0852f73b20da0208c7e627975a59ff072379883d8457d03104651032033d"
        );
    }

    #[tokio::test]
    async fn test_proxy() {
        dotenv().ok();

        let proxy_url: Url = Url::parse(PROXY_URL).unwrap();

        let proxy_username = dotenv::var("PROXY_USERNAME").unwrap();
        let proxy_password = dotenv::var("PROXY_PASSWORD").unwrap();

        let proxy = reqwest::Proxy::all(proxy_url.clone())
            .unwrap()
            .basic_auth(&proxy_username, &proxy_password);

        let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();

        let res = http_client
            .get("https://httpbin.org/ip")
            .send()
            .await
            .unwrap();

        let status = res.status();
        assert_eq!(status.as_u16(), 200);

        #[derive(Deserialize)]
        struct Ip {
            origin: String,
        }

        let ip: Ip = res.json().await.unwrap();
        assert_eq!(ip.origin, PROXY_IP);
    }
}
