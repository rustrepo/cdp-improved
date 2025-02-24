use futures::StreamExt;
use chromiumoxide::Browser;
use tokio;
use reqwest; // Add this import for making HTTP requests
use serde_json::Value; // Add this import for parsing JSON


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   
// Query the browser's debugging endpoint to get the list of targets
let debug_url = "http://localhost:9222/json";
let response = reqwest::get(debug_url).await?.json::<Vec<Value>>().await?;

// Use the WebSocket URL of the first target
let ws_url = response
    .get(0) // Get the first target
    .and_then(|target| target["webSocketDebuggerUrl"].as_str())
    .ok_or("No valid WebSocket URL found")?;

    //println!("Connecting to WebSocket URL: {}", ws_url);

    // Connect to the browser using the WebSocket URL
    let (browser, mut handler) = Browser::connect(ws_url).await?;

    // Spawn a task to handle browser events
    let _handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                eprintln!("Handler encountered an error: {:?}", event.unwrap_err());
                break;
            }
        }
    });

    // Open a new tab with the desired URL
    let desired_url = "https://www.rust-lang.org/learn";
    let page = browser.new_page(desired_url).await?;

    // Wait for the page to load completely
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
        Ok(_) => println!("Page closed successfully."),
        Err(e) => {
            if e.to_string().contains("Not attached to an active page") {
                println!("Page is already closed.");
            } else {
                eprintln!("Failed to close the page: {}", e);
            }
        }
    }


    println!("Done!");
    Ok(())
}