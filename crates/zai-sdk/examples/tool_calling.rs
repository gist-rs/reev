//! Tool calling example using the ZAI SDK
//!
//! This example demonstrates how to use the SDK for function calling with GLM-4.6.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use zai_sdk::models::ToolDefinition;
use zai_sdk::{GlmVariant, Message, ZaiClient};

// Define tools for the model to use
#[derive(Debug, Deserialize)]
struct GetWeatherArgs {
    location: String,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetStockPriceArgs {
    symbol: String,
}

// Example tool responses
#[derive(Serialize)]
struct WeatherResponse {
    location: String,
    temperature: f64,
    unit: String,
    condition: String,
}

#[derive(Serialize)]
struct StockPriceResponse {
    symbol: String,
    price: f64,
    currency: String,
}

// Mock tool implementations
fn get_weather(args: GetWeatherArgs) -> WeatherResponse {
    // In a real implementation, this would call a weather API
    WeatherResponse {
        location: args.location,
        temperature: 22.5,
        unit: args.unit.unwrap_or_else(|| "celsius".to_string()),
        condition: "Partly cloudy".to_string(),
    }
}

fn get_stock_price(args: GetStockPriceArgs) -> StockPriceResponse {
    // In a real implementation, this would call a stock API
    let prices: HashMap<String, f64> = HashMap::from([
        ("AAPL".to_string(), 178.23),
        ("GOOGL".to_string(), 142.56),
        ("MSFT".to_string(), 412.89),
    ]);

    let symbol = args.symbol.clone();
    let price = *prices.get(&symbol).unwrap_or(&0.0);

    StockPriceResponse {
        symbol,
        price,
        currency: "USD".to_string(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    // Create a client with the builder pattern
    let client = ZaiClient::builder()
        .variant(GlmVariant::Standard)
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("Tool calling example:");
    println!("-------------------");

    // Define tools for the model to use
    let get_weather_tool = ToolDefinition::function(
        "get_weather",
        "Get the current weather information for a specific location",
    )
    .parameter(
        "location",
        "string",
        "The city name, e.g., 'San Francisco, CA'",
        true,
    )
    .parameter(
        "unit",
        "string",
        "The unit of temperature, either 'celsius' or 'fahrenheit'",
        false,
    )
    .build();

    let get_stock_price_tool = ToolDefinition::function(
        "get_stock_price",
        "Get the current stock price for a specific symbol",
    )
    .parameter(
        "symbol",
        "string",
        "The stock ticker symbol, e.g., 'AAPL'",
        true,
    )
    .build();

    // Create a request with tools
    let request = zai_sdk::CompletionRequest::new("glm-4.6")
        .system("You are a helpful assistant with access to weather and stock information. Use the appropriate tools when needed.")
        .user("What's the weather like in New York and what's the current price of Apple stock?")
        .tool(get_weather_tool.into())
        .tool(get_stock_price_tool.into())
        .temperature(0.1)
        .max_tokens(500);

    // Send the request
    let response = client.send_completion_request(request).await?;

    println!("Model response:");
    if let Some(content) = response.content() {
        println!("{content}");
    }

    // Check if the model wants to call any tools
    if let Some(_choice) = response.choices.first() {
        // In a real implementation, you would check for tool calls in the response
        // For this example, we'll simulate a tool call response
        println!("\nSimulated tool calls:");

        // Mock weather tool call
        let weather_args = GetWeatherArgs {
            location: "New York".to_string(),
            unit: Some("fahrenheit".to_string()),
        };
        let weather = get_weather(weather_args);
        println!(
            "Weather in {}: {}°{} ({})",
            weather.location, weather.temperature, weather.unit, weather.condition
        );

        // Mock stock price tool call
        let stock_args = GetStockPriceArgs {
            symbol: "AAPL".to_string(),
        };
        let stock = get_stock_price(stock_args);
        println!(
            "{} stock price: {} {}",
            stock.symbol, stock.price, stock.currency
        );

        // Send the tool results back to the model
        let follow_up_request = zai_sdk::CompletionRequest::new("glm-4.6")
            .message(Message::system(
                "You are a helpful assistant with access to weather and stock information.",
            ))
            .message(Message::user(
                "What's the weather like in New York and what's the current price of Apple stock?",
            ))
            .message(Message::assistant(
                "I'll check the weather in New York and get the current Apple stock price for you.",
            ))
            // In a real implementation, these would be the actual tool results from the model
            .message(Message::tool(format!(
                "The weather in New York is {}°{} and {}.",
                weather.temperature, weather.unit, weather.condition
            )))
            .message(Message::tool(format!(
                "The current price of Apple stock (AAPL) is {} {}.",
                stock.price, stock.currency
            )))
            .user("Can you summarize this information for me?")
            .temperature(0.1)
            .max_tokens(200);

        let follow_up_response = client.send_completion_request(follow_up_request).await?;

        println!("\nModel summary:");
        if let Some(content) = follow_up_response.content() {
            println!("{content}");
        }
    }

    Ok(())
}
