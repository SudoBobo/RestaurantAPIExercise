#[cfg(test)]
mod tests {
    use tokio::task;
    use reqwest;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use rocket::serde::json::json;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use crate::{create_rocket, rocket, ErrorResponse};

    #[derive(Serialize, Deserialize, Debug)]
    struct Order {
        item_id: String,
        table_id: String,
    }

    #[derive(Serialize, PartialEq, Deserialize, Debug)]
    #[serde(crate = "rocket::serde")]
    struct OrderResult {
        order_id: String,
        item_id: String,
        table_id: String,
        cooking_time: i32,
    }

    #[test]
    fn put_empty_body() {
        let rocket = create_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.put("/order/123")
            .header(ContentType::JSON)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);

        let error_response: ErrorResponse = response.into_json().unwrap();
        assert_eq!(error_response.error_code, "INVALID_BODY");
    }

    #[test]
    fn put_non_json_content_type() {
        let rocket = create_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.put("/order/123")
            .header(ContentType::Plain)
            .body("item_id=1&table_id=2")
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn put_wrong_http_method() {
        let rocket = create_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post("/order/123")
            .header(ContentType::JSON)
            .body(json!({"item_id": "1", "table_id": "2"}).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn put_body_with_missing_params() {
        let rocket = create_rocket();
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.put("/order/123")
            .header(ContentType::JSON)
            .body(json!({"table_id": "2"}).to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);

        let error_response: ErrorResponse = response.into_json().unwrap();
        assert_eq!(error_response.error_code, "INVALID_BODY");
    }

    #[test]
    fn put_order_happy_path() {
        let client = Client::new(rocket()).unwrap();
        let uuid = Uuid::new_v4();
        let res = client
            .put(format!("/order/{}", uuid))
            .json(&Order {
                item_id: String::from("123"),
                table_id: String::from("1"),
            })
            .dispatch();

        let status = res.status();
        let order_result: OrderResult = res.into_json().unwrap();

        assert_eq!(status, Status::Ok);
        assert_eq!(order_result.order_id, uuid.to_string());
        assert_eq!(order_result.item_id, "123");
        assert_eq!(order_result.table_id, "1");
        assert!((5..=15).contains(&order_result.cooking_time));
    }

    #[test]
    fn put_duplicate_order() {
        let client = Client::new(rocket()).unwrap();
        let uuid = Uuid::new_v4().to_string();
        let order = Order {
            item_id: "123".to_string(),
            table_id: "1".to_string(),
        };

        let res = client
            .put(format!("/order/{}", uuid))
            .json(&order)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // Second PUT request with the same ID
        let res = client
            .put(format!("/order/{}", uuid))
            .header(ContentType::JSON)
            .body(json!(order).to_string())
            .dispatch();

        assert_eq!(res.status(), Status::Conflict);

        let error_response: ErrorResponse = res.into_json().unwrap();
        assert_eq!(error_response.error_code, "DUPLICATE_ORDER");
    }

    #[test]
    fn get_orders_by_table() {
        let client = Client::new(rocket()).unwrap();

        for i in 301..304 {
            let uuid = Uuid::new_v4();
            client
                .put(format!("/order/{}", uuid))
                .json(&Order {
                    item_id: i.to_string(),
                    table_id: String::from("3"),
                })
                .dispatch();
        }

        let res = client.get("/orders?table_id=3").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let orders: Vec<OrderResult> = res.into_json().unwrap();

        assert_eq!(orders.len(), 3);
        let item_ids: Vec<String> = orders.iter().map(|o| o.item_id.clone()).collect();
        for i in 301..304 {
            assert!(item_ids.contains(&i.to_string()));
        }
    }

    #[test]
    fn get_orders_by_table_and_item() {
        let client = Client::new(rocket()).unwrap();
        let uuid = Uuid::new_v4();

        let res = client
            .put(format!("/order/{}", uuid))
            .json(&Order {
                item_id: String::from("401"),
                table_id: String::from("4"),
            })
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client
            .get("/orders?table_id=4&item_id=401")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let orders: Vec<OrderResult> = res.into_json().unwrap();

        assert!(!orders.is_empty());

        let order = orders
            .iter()
            .find(|o| o.item_id == "401" && o.table_id == "4")
            .expect("Order not found");

        assert!((5..=15).contains(&order.cooking_time));
    }

    #[test]
    fn delete_item_from_table() {
        let client = Client::new(rocket()).unwrap();
        let uuid = Uuid::new_v4();

        let res = client
            .put(format!("/order/{}", uuid))
            .json(&Order {
                item_id: String::from("201"),
                table_id: String::from("2"),
            })
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client
            .delete(format!("/order/{}", uuid))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client.get("/orders?table_id=2").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let orders: Vec<OrderResult> = res.into_json().unwrap();

        assert!(orders.is_empty());
    }

    #[test]
    fn delete_nonexistent_order() {
        let client = Client::new(rocket()).unwrap();        let uuid = Uuid::new_v4();

        let res = client
            .delete(format!("/order/{}", uuid))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);

        let error_response: ErrorResponse = res.into_json().unwrap();
        assert_eq!(error_response.error_code, "ORDER_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_concurrent_put_order() {
        let rocket = create_rocket();

        let server = rocket.ignite().await.unwrap();
        let shutdown_handle = server.shutdown();

        let server_handle = tokio::spawn(async move {
            server.launch().await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut handles = vec![];
        for thread_num in 0..10 {
            let handle = task::spawn(async move {
                let client = reqwest::Client::new();
                let start_time = std::time::Instant::now();
                let mut iteration_num = 0;
                while start_time.elapsed() < std::time::Duration::from_secs(10) {
                    // Send 3 PUT requests
                    let mut order_ids = Vec::new();
                    for order_num in 0..3 {
                        let order_id = format!("order{}_{}_{}", thread_num, iteration_num, order_num);
                        order_ids.push(order_id.clone());
                        let order = Order {
                            item_id: format!("item{}", thread_num),
                            table_id: format!("table{}", thread_num % 3),
                        };
                        let res = client
                            .put(&format!("http://localhost:8000/order/{}", order_id))
                            .json(&order)
                            .send()
                            .await
                            .expect("Failed to send PUT request");
                        assert!(res.status().is_success());
                    }

                    // Query based on item
                    let item_id = format!("item{}", thread_num);
                    let res = client
                        .get(&format!("http://localhost:8000/orders?item_id={}", item_id))
                        .send()
                        .await
                        .expect("Failed to send GET request");
                    assert!(res.status().is_success());
                    let orders: Vec<OrderResult> = res.json().await.expect("Failed to parse response");
                    assert!(orders.len() >= 3); // At least the ones we just inserted

                    // Query based on table
                    let table_id = format!("table{}", thread_num % 3);
                    let res = client
                        .get(&format!("http://localhost:8000/orders?table_id={}", table_id))
                        .send()
                        .await
                        .expect("Failed to send GET request");
                    assert!(res.status().is_success());
                    let orders_by_table: Vec<OrderResult> = res.json().await.expect("Failed to parse response");

                    // Delete all three items
                    for order_id in &order_ids {
                        let res = client
                            .delete(&format!("http://localhost:8000/order/{}", order_id))
                            .send()
                            .await
                            .expect("Failed to send DELETE request");
                        assert!(res.status().is_success());
                    }
                    // Sleep 100 ms
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    iteration_num += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        shutdown_handle.notify();
        server_handle.await.unwrap();
    }
}
