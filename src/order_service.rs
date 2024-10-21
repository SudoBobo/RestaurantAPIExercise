use std::sync::RwLock;
use std::collections::HashMap;
use rocket::serde::{Deserialize, Serialize};
use std::fmt;
use std::error::Error;
use rand::Rng;

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Order {
    pub item_id: String,
    pub table_id: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct OrderResult {
    pub order_id: String,
    pub item_id: String,
    pub table_id: String,
    pub cooking_time: i32,
}

#[derive(Debug)]
pub enum OrderServiceError {
    DuplicateOrder(String),
    OrderNotFound(String),
    MutexPoisoned(String),
}

impl fmt::Display for OrderServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderServiceError::DuplicateOrder(id) => write!(f, "Order with id '{}' already exists.", id),
            OrderServiceError::OrderNotFound(id) => write!(f, "Order with id '{}' not found.", id),
            OrderServiceError::MutexPoisoned(msg) => write!(f, "Mutex poisoned: {}", msg),
        }
    }
}

impl Error for OrderServiceError {}

// OrderService provides an abstract way to create, delete, and fetch orders.
// We can do unit testing on our endpoints by providing a mock implementation of OrderService.
// We can also easily switch between in-memory and on-disk (DB) implementations.
pub trait OrderService: Sync + Send {
    fn put_order(&self, id: String, order: Order) -> Result<OrderResult, OrderServiceError>;
    fn delete_order(&self, order_id: String) -> Result<OrderResult, OrderServiceError>;
    fn get_orders(&self, table_id: Option<String>, item_id: Option<String>) -> Result<Vec<OrderResult>, OrderServiceError>;
}

// InMemoryOrderService stores orders in memory using HashMaps wrapped in RwLock for thread safety.
pub struct InMemoryOrderService {
    orders: RwLock<HashMap<String, OrderResult>>,
    tables_idx: RwLock<HashMap<String, Vec<String>>>,
}

pub fn new_in_memory() -> InMemoryOrderService {
    InMemoryOrderService {
        orders: RwLock::new(HashMap::new()),
        tables_idx: RwLock::new(HashMap::new()),
    }
}

impl OrderService for InMemoryOrderService {
    fn put_order(&self, id: String, order: Order) -> Result<OrderResult, OrderServiceError> {
        let mut orders = self.orders.write()
            .map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain write mutex".into()))?;

        if orders.contains_key(&id) {
            return Err(OrderServiceError::DuplicateOrder(id));
        }

        let order_result = OrderResult {
            order_id: id.clone(),
            item_id: order.item_id.clone(),
            table_id: order.table_id.clone(),
            cooking_time: rand::thread_rng().gen_range(5..16),
        };

        orders.insert(id.clone(), order_result.clone());

        let mut tables_idx = self.tables_idx.write()
            .map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain write mutex".into()))?;
        tables_idx.entry(order.table_id.clone()).or_insert_with(Vec::new).push(id);

        Ok(order_result)
    }

    fn delete_order(&self, order_id: String) -> Result<OrderResult, OrderServiceError> {
        let mut orders = self.orders.write()
            .map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain write mutex".into()))?;

        let order = orders.remove(&order_id)
            .ok_or(OrderServiceError::OrderNotFound(order_id.clone()))?;

        let mut tables_idx = self.tables_idx.write()
            .map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain write mutex".into()))?;
        if let Some(table) = tables_idx.get_mut(&order.table_id) {
            table.retain(|x| x != &order_id);
        }

        Ok(order)
    }

    fn get_orders(
        &self,
        table_id: Option<String>,
        item_id: Option<String>,
    ) -> Result<Vec<OrderResult>, OrderServiceError> {
        let orders = self.orders.read().
            map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain read mutex".into()))?;
        let tables_idx = self.tables_idx.read()
            .map_err(|_| OrderServiceError::MutexPoisoned("Failed to obtain read mutex".into()))?;

        let result = match (table_id, item_id) {
            (None, None) => orders.values().cloned().collect(),
            (Some(table_id), None) => {
                if let Some(order_ids) = tables_idx.get(&table_id) {
                    order_ids
                        .iter()
                        .filter_map(|id| orders.get(id).cloned())
                        .collect()
                } else {
                    Vec::new()
                }
            }
            (Some(table_id), Some(item_id)) => {
                if let Some(order_ids) = tables_idx.get(&table_id) {
                    order_ids
                        .iter()
                        .filter_map(|id| {
                            orders
                                .get(id)
                                .filter(|order| order.item_id == item_id)
                                .cloned()
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            (None, Some(item_id)) => orders
                .values()
                .filter(|order| order.item_id == item_id)
                .cloned()
                .collect(),
        };
        Ok(result)
    }
}
