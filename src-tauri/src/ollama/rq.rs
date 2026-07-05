use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct OllamaModel {
    name: String,
}

#[derive(Deserialize, Debug)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[tauri::command]
pub async fn fetch_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get("http://localhost:11434/api/tags").send().await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let tags: OllamaTagsResponse = response.json().await.map_err(|e| e.to_string())?;
                let model_names = tags.models.into_iter().map(|m| m.name).collect();
                Ok(model_names)
            } else {
                Err("Ollama returned an error status".to_string())
            }
        }
        Err(_) => Err("Could not connect to Ollama to fetch models".to_string()),
    }
}

// Sends the text and returns the complete formatted string response once finished
#[tauri::command]
pub async fn process_brain_dump(selected_model: String, raw_dump: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let system_prompt = String::from(
        "You are FeatherNotes, a precise organization assistant. \
        Analyze the user's raw, unstructured brain dump and categorize the information into exactly 3 sections: \
        1. TASKS (Actionable items with estimated priority) \
        2. KNOWLEDGE (Facts, ideas, or notes for long-term storage) \
        3. MOOD ANALYSIS (A quick assessment of the user's current mental state or stress level based on the text) \
        Be concise, use clean formatting, and do not add any conversational filler introduction or conclusion."
    );

    let payload = OllamaGenerateRequest {
        model: selected_model,
        prompt: raw_dump,
        system: system_prompt,
        stream: false, // Disabling stream to wait for complete output
    };

    let res = client.post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if res.status().is_success() {
        let json_res: OllamaGenerateResponse = res.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(json_res.response)
    } else {
        Err(format!("Ollama API error status: {}", res.status()))
    }
}