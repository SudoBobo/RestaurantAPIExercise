#[cfg(test)]
mod tests {
    use crate::order_service::{new_in_memory, InMemoryOrderService, Order, OrderService, OrderServiceError};

    fn setup_service() -> InMemoryOrderService {
        new_in_memory()
    }

    #[test]
    fn test_put_order_success() {
        let service = setup_service();
        let order = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };

        let result = service.put_order("order1".to_string(), order);
        assert!(result.is_ok());
        let order_result = result.unwrap();
        assert_eq!(order_result.order_id, "order1");
        assert_eq!(order_result.item_id, "item1");
        assert_eq!(order_result.table_id, "table1");
    }

    #[test]
    fn test_put_duplicate_order_fails() {
        let service = setup_service();
        let order = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };

        let result = service.put_order("order1".to_string(), order.clone());
        assert!(result.is_ok());

        let duplicate_result = service.put_order("order1".to_string(), order);
        assert!(duplicate_result.is_err());

        if let Err(OrderServiceError::DuplicateOrder(order_id)) = duplicate_result {
            assert_eq!(order_id, "order1");
        } else {
            panic!("Expected DuplicateOrder error.");
        }
    }

    #[test]
    fn test_delete_order_success() {
        let service = setup_service();
        let order = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };

        let result = service.put_order("order1".to_string(), order);
        assert!(result.is_ok());

        let delete_result = service.delete_order("order1".to_string());
        assert!(delete_result.is_ok());
        let deleted_order = delete_result.unwrap();
        assert_eq!(deleted_order.order_id, "order1");
    }

    #[test]
    fn test_delete_order_not_found() {
        let service = setup_service();

        let delete_result = service.delete_order("non_existent_order".to_string());
        assert!(delete_result.is_err());

        if let Err(OrderServiceError::OrderNotFound(order_id)) = delete_result {
            assert_eq!(order_id, "non_existent_order");
        } else {
            panic!("Expected OrderNotFound error.");
        }
    }

    #[test]
    fn test_get_orders_by_table_id() {
        let service = setup_service();
        let order1 = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };
        let order2 = Order {
            item_id: "item2".to_string(),
            table_id: "table1".to_string(),
        };

        service.put_order("order1".to_string(), order1).unwrap();
        service.put_order("order2".to_string(), order2).unwrap();

        let orders = service.get_orders(Some("table1".to_string()), None).unwrap();
        assert_eq!(orders.len(), 2);
    }

    #[test]
    fn test_get_orders_by_item_id() {
        let service = setup_service();
        let order1 = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };
        let order2 = Order {
            item_id: "item2".to_string(),
            table_id: "table1".to_string(),
        };

        service.put_order("order1".to_string(), order1).unwrap();
        service.put_order("order2".to_string(), order2).unwrap();

        let orders = service.get_orders(None, Some("item1".to_string())).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].item_id, "item1");
    }

    #[test]
    fn test_get_orders_by_table_and_item_id() {
        let service = setup_service();
        let order1 = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };
        let order2 = Order {
            item_id: "item2".to_string(),
            table_id: "table1".to_string(),
        };

        service.put_order("order1".to_string(), order1).unwrap();
        service.put_order("order2".to_string(), order2).unwrap();

        let orders = service.get_orders(Some("table1".to_string()), Some("item1".to_string())).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].item_id, "item1");
    }

    #[test]
    fn test_get_all_orders() {
        let service = setup_service();
        let order1 = Order {
            item_id: "item1".to_string(),
            table_id: "table1".to_string(),
        };
        let order2 = Order {
            item_id: "item2".to_string(),
            table_id: "table2".to_string(),
        };

        service.put_order("order1".to_string(), order1).unwrap();
        service.put_order("order2".to_string(), order2).unwrap();

        let orders = service.get_orders(None, None).unwrap();
        assert_eq!(orders.len(), 2);
    }
}
