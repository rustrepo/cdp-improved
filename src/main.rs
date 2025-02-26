/*
[dependencies]
scraper = "0.12"
tokio = { version = "1", features = ["full"] }
chromiumoxide = "0.5"
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
futures = "0.3"  # Add this line
dotenv = "0.15.0"
*/


use futures::StreamExt;
use chromiumoxide::Browser;
use tokio;
use reqwest; // For making HTTP requests
use serde_json::Value; // For parsing JSON
use dotenv::dotenv;
use std::env;
use tokio::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let desired_url = env::var("URL").expect("BASE_URL not found in .env file");

    // Query the browser's debugging endpoint to get the list of targets
    let debug_url = "http://localhost:9222/json";
    let response = reqwest::get(debug_url).await?.json::<Vec<Value>>().await?;

    // Use the WebSocket URL of the first target
    let ws_url = response
        .get(0) // Get the first target
        .and_then(|target| target["webSocketDebuggerUrl"].as_str())
        .ok_or("No valid WebSocket URL found")?;

    // Connect to the browser using the WebSocket URL
    let (browser, mut handler) = Browser::connect(ws_url).await?;

    // Spawn a task to handle browser events
    let _handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(_e) = event {
                // Log the error but continue processing
            }
        }
    });

    // Open a new blank page
    let page = browser.new_page("about:blank").await?;

    // Fetch the WebSocket URL of the newly opened page
    let debug_url = "http://localhost:9222/json";
    let response = reqwest::get(debug_url).await?.json::<Vec<Value>>().await?;

    // Find the WebSocket URL of the newly opened page
    let new_ws_url = response
        .iter()
        .find(|target| target["url"].as_str() == Some("about:blank"))
        .and_then(|target| target["webSocketDebuggerUrl"].as_str())
        .ok_or("No valid WebSocket URL found for the new page")?;

    // Connect to the new WebSocket URL
    let (_new_browser, mut new_handler) = Browser::connect(new_ws_url).await?;

    // Spawn a task to handle browser events for the new connection
    let _new_handle = tokio::spawn(async move {
        while let Some(event) = new_handler.next().await {
            if let Err(_e) = event {
                // Log the error but continue processing
            }
        }
    });

        // Navigate to the desired URL on the new page
        page.goto(&desired_url).await?;
        page.wait_for_navigation().await?;

       // Find the paragraph element
    let paragraph = page
    .find_element("p.pl4-l.ma0-l.mt1.mb3")
    .await
    .map_err(|e| format!("Failed to find the paragraph element: {}", e))?;

// Extract the text content of the paragraph
let paragraph_text = paragraph.inner_text().await?;

// Handle the Option<String> returned by inner_text
if let Some(text) = paragraph_text {
    println!("Paragraph text: {}", text);

    // Compare the paragraph text with a fixed string
    let fixed_text = "Your fixed text here";
    if text == fixed_text {
        println!("The paragraph text matches the fixed text.");
    } else {
        println!("The paragraph text does not match the fixed text.");
    }
} else {
    println!("The paragraph element has no text content.");
}

// Use JavaScript to set the dropdown value to "it" (Italian)
let dropdown_selector = "select#language-nav";
let js_code = format!(
    "document.querySelector('{}').value = 'it';",
    dropdown_selector
);
page.evaluate(js_code).await?;

println!("Switched to Italian (it).");

// Verify the selected value using JavaScript
let selected_value = page
    .evaluate(format!(
        "document.querySelector('{}').value",
        dropdown_selector
    ))
    .await?
    .into_value::<String>()?;

println!("Selected value: {}", selected_value);

// Click the button/link element
let button_selector = "a.button.button-secondary";
let js_code = format!(
    "document.querySelector('{}').click();",
    button_selector
);
page.evaluate(js_code).await?;

println!("Button clicked successfully in the background.");

// Wait for navigation to complete (if the button triggers navigation)
page.wait_for_navigation().await?;
        

       
    // Attempt to close the page and handle the error if it's already closed
    match page.close().await {
        Ok(_) => println!("\nPage closed successfully."),
        Err(e) => {
            if e.to_string().contains("Not attached to an active page") {
                // Page is already closed
            } else {
                eprintln!("Failed to close the page: {}", e);
            }
        }
    }

    Ok(())
}
