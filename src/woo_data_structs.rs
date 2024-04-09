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

// https://docs.woo.org/#get-orders
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct GetOrder {
    pub symbol: Option<String>,
    pub side: Option<String>,
    pub size: Option<u32>,
    pub order_type: Option<String>,
    pub order_tag: Option<String>,
    pub realized_pnl: Option<bool>,
    pub status: Option<String>,
    pub start_t: Option<u64>,
    pub end_t: Option<u64>,
    pub page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOrderRes {
    pub success: bool,
    pub meta: Meta,
    pub rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub total: u32,
    pub records_per_page: u32,
    pub current_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Row {
    side: String,
    status: String,
    symbol: String,
    client_order_id: Option<u32>,
    reduce_only: bool,
    order_id: u32,
    order_tag: String,
    r#type: String,
    price: f64,
    quantity: f64,
    amount: Option<f64>,
    visible: f64,
    executed: f64,
    total_fee: f64,
    fee_asset: Option<String>,
    total_rebate: Option<f64>,
    rebate_asset: Option<String>,
    created_time: String,
    updated_time: String,
    average_executed_price: Option<f64>,
    position_side: String,
    realized_pnl: Option<f64>,
}
