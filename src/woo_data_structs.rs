use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct WooOrder {
    pub symbol: String,
    pub client_order_id: Option<u32>,
    pub order_tag: Option<String>,
    pub order_type: String,
    pub order_price: Option<f64>,
    pub order_quantity: Option<f64>,
    pub order_amount: Option<f64>,
    pub reduce_only: Option<bool>,
    pub visible_quantity: Option<f64>,
    pub side: String,
    pub position_side: Option<String>,
}

#[derive(Serialize)]
pub struct CancelOrder {
    pub order_id: u32,
    pub symbol: String,
}

#[derive(Deserialize)]
pub struct CancelOrderRes {
    pub success: bool,
    pub status: String,
}
#[derive(Deserialize)]
pub struct SendOrderRes {
    pub success: bool,
    pub timestamp: String,
    pub order_id: u32,
    pub order_type: String,
    pub client_order_id: u32,
    pub order_price: Option<f64>,
    pub order_quantity: Option<f64>,
    pub order_amount: Option<f64>,
    pub reduce_only: Option<bool>,
}
