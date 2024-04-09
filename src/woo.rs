use crate::constants::{PROXY_IP, PROXY_URL, WOO_API_BASE_URL, WOO_API_BASE_URL_STAGING};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use url::Url;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
struct WooOrder {
    symbol: String,
    client_order_id: Option<u32>,
    order_tag: Option<String>,
    order_type: String,
    order_price: Option<f64>,
    order_quantity: Option<f64>,
    order_amount: Option<f64>,
    reduce_only: Option<bool>,
    visible_quantity: Option<f64>,
    side: String,
    position_side: Option<String>,
}

struct Woo;

impl Woo {
    fn generate_sorted_query_string<P>(body: P) -> String
    where
        P: Serialize,
    {
        let unsorted_query_string =
            serde_qs::to_string(&body).expect("fail to serialize to query string");

        let mut sorted_query_string = unsorted_query_string.split('&').collect::<Vec<&str>>();
        sorted_query_string.sort();

        let a = sorted_query_string.join("&");
        // println!("{:?}", a);
        a
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
        let url = format!("{}v1/public/system_info", WOO_API_BASE_URL);
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
            .get(woo_url)
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

        println!("{:?}", response.text().await);
    }

    #[tokio::test]
    async fn send_order() {
        dotenv().ok();

        let mut woo_url: Url = Url::parse(WOO_API_BASE_URL_STAGING).unwrap();
        woo_url.set_path("v1/order");

        let woo_api_key = dotenv::var("WOO_API_KEY_STAGING").unwrap();
        let woo_api_secret = dotenv::var("WOO_API_SECRET_STAGING").unwrap();

        let proxy_url: Url = Url::parse(PROXY_URL).unwrap();

        let proxy_username = dotenv::var("PROXY_USERNAME").unwrap();
        let proxy_password = dotenv::var("PROXY_PASSWORD").unwrap();

        let timestamp = chrono::Utc::now().timestamp_millis();

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

        // this part is to handle the alphabetical order of the query string
        // `url_encoded` is just an intermediate step
        let url_encoded = serde_qs::to_string(&order).expect("failed to serialize to query string");
        let deserialized: BTreeMap<String, String> = serde_qs::from_str(&url_encoded).unwrap();

        let proxy = reqwest::Proxy::all(proxy_url.clone())
            .unwrap()
            .basic_auth(&proxy_username, &proxy_password);

        let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();

        let request = http_client
            .post(woo_url)
            .header("x-api-key", woo_api_key)
            .header("x-api-timestamp", timestamp)
            .header(
                "x-api-signature",
                Woo::generate_hmac_sha256_signature(
                    Woo::generate_sorted_query_string(&order),
                    timestamp as u64,
                    woo_api_secret.to_string(),
                ),
            )
            .form(&deserialized);

        let response = request.send().await.expect("failed to send request");

        assert_eq!(response.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn cancel_order() {
        dotenv().ok();

        let mut woo_url: Url = Url::parse(WOO_API_BASE_URL_STAGING).unwrap();
        woo_url.set_path("v1/order");

        let woo_api_key = dotenv::var("WOO_API_KEY_STAGING").unwrap();
        let woo_api_secret = dotenv::var("WOO_API_SECRET_STAGING").unwrap();

        let proxy_url: Url = Url::parse(PROXY_URL).unwrap();

        let proxy_username = dotenv::var("PROXY_USERNAME").unwrap();
        let proxy_password = dotenv::var("PROXY_PASSWORD").unwrap();

        let timestamp = chrono::Utc::now().timestamp_millis();

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

        // this part is to handle the alphabetical order of the query string
        // `url_encoded` is just an intermediate step
        let url_encoded = serde_qs::to_string(&order).expect("failed to serialize to query string");
        let deserialized: BTreeMap<String, String> = serde_qs::from_str(&url_encoded).unwrap();

        let proxy = reqwest::Proxy::all(proxy_url.clone())
            .unwrap()
            .basic_auth(&proxy_username, &proxy_password);

        let http_client = reqwest::Client::builder().proxy(proxy).build().unwrap();

        let request = http_client
            .post(woo_url)
            .header("x-api-key", woo_api_key)
            .header("x-api-timestamp", timestamp)
            .header(
                "x-api-signature",
                Woo::generate_hmac_sha256_signature(
                    Woo::generate_sorted_query_string(&order),
                    timestamp as u64,
                    woo_api_secret.to_string(),
                ),
            )
            .form(&deserialized);

        let response = request.send().await.expect("failed to send request");

        assert_eq!(response.status().as_u16(), 200);
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
