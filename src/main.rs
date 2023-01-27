use lambda_http::{
    aws_lambda_events::serde_json::json, run, service_fn, Body, Error, IntoResponse, Request,
    RequestExt, Response,
};
use serde::Serialize;

struct PizzaList {
    pizzas: Vec<Pizza>,
}

#[derive(Serialize)]
struct Pizza {
    name: String,
    price: i32,
}

impl PizzaList {
    fn new() -> PizzaList {
        PizzaList {
            pizzas: vec![
                Pizza {
                    name: String::from("veggie"),
                    price: 10,
                },
                Pizza {
                    name: String::from("regina"),
                    price: 12,
                },
                Pizza {
                    name: String::from("deluxe"),
                    price: 14,
                },
            ],
        }
    }
}

fn get_pizza_from_name<'a>(pizza_name: &'a str, pizza_list: &'a PizzaList) -> Option<&'a Pizza> {
    let mut iter = pizza_list.pizzas.iter();
    iter.find(|pizza| pizza.name == pizza_name)
}

async fn build_success_response(pizza: &Pizza) -> Response<Body> {
    json!(pizza).into_response().await
}

async fn build_failure_response(msg: &str) -> Response<Body> {
    Response::builder()
        .status(400)
        .header("content-type", "application/json")
        .body(Body::from(json!({ "error": msg }).to_string()))
        .expect("could not build the error response")
}

fn process_event<'a>(
    pizza_name: Option<&'a str>,
    pizza_list: &'a PizzaList,
) -> Result<&'a Pizza, &'a str> {
    match pizza_name {
        Some(pizza_name) => match get_pizza_from_name(pizza_name, pizza_list) {
            Some(pizza) => Ok(pizza),
            _ => Err("no pizza found for the given name"),
        },
        _ => Err("could not find the pizza name"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let all_pizza = &PizzaList::new();
    let handler_func = |event: Request| async move {
        let response = match process_event(event.path_parameters().first("pizza_name"), all_pizza)
        {
            Ok(pizza) => build_success_response(pizza).await,
            Err(error_msg) => build_failure_response(error_msg).await,
        };
        Result::<Response<Body>, Error>::Ok(response)
    };
    run(service_fn(handler_func)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_pizza_list_test() {
        let all_pizza = PizzaList::new();
        assert_eq!(all_pizza.pizzas.len(), 3);
        let veggie = get_pizza_from_name("veggie", &all_pizza);
        assert_eq!(10, veggie.unwrap().price);
        let regina = get_pizza_from_name("regina", &all_pizza);
        assert_eq!(12, regina.unwrap().price);
        let deluxe = get_pizza_from_name("deluxe", &all_pizza);
        assert_eq!(14, deluxe.unwrap().price);
    }
    #[tokio::test]
    async fn build_success_responsee_test() {
        let test_pizza = Pizza {
            name: String::from("test_pizza"),
            price: 100,
        };
        let result = build_success_response(&test_pizza).await;
        let (parts, body) = result.into_parts();
        assert_eq!(200, parts.status);
        assert_eq!(
            "application/json",
            parts.headers.get("content-type").unwrap()
        );
        assert_eq!(
            "{\"name\":\"test_pizza\",\"price\":100}",
            String::from_utf8(body.to_ascii_lowercase()).unwrap()
        )
    }

    #[tokio::test]
    async fn build_failure_responsee_test() {
        let result = build_failure_response("test error message").await;
        let (parts, body) = result.into_parts();
        assert_eq!(400, parts.status);
        assert_eq!(
            "application/json",
            parts.headers.get("content-type").unwrap()
        );
        assert_eq!(
            "{\"error\":\"test error message\"}",
            String::from_utf8(body.to_ascii_lowercase()).unwrap()
        )
    }

    #[test]
    fn process_event_valid_pizza_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(Some("regina"), &pizza_list);
        assert!(res.is_ok());
    }

    #[test]
    fn process_event_invalid_pizza_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(Some("unknown pizza"), &pizza_list);
        assert!(matches!(res, Err("no pizza found for the given name")));
    }

    #[test]
    fn process_event_no_pizza_test() {
        let pizza_list = PizzaList::new();
        let res = process_event(None, &pizza_list);
        assert!(matches!(res, Err("could not find the pizza name")));
    }
}
