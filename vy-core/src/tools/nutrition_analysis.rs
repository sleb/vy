use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug)]
pub struct NutritionAnalysisError(String);

impl std::fmt::Display for NutritionAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for NutritionAnalysisError {}

impl NutritionAnalysisError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct NutritionAnalysisArgs {
    pub image_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngredientEstimate {
    pub ingredient: String,
    pub amount_grams: f32,
    pub confidence: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NutritionAnalysisResponse {
    pub ingredients: Vec<IngredientEstimate>,
    pub summary: String,
    pub notes: Option<String>,
}

impl fmt::Display for NutritionAnalysisResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🍽️ **Nutrition Analysis Results**")?;
        writeln!(f)?;

        for ingredient in &self.ingredients {
            writeln!(
                f,
                "• **{}** - {}g ({})",
                ingredient.ingredient, ingredient.amount_grams, ingredient.confidence
            )?;
        }

        writeln!(f)?;
        writeln!(f, "**Summary:** {}", self.summary)?;

        if let Some(notes) = &self.notes {
            if !notes.trim().is_empty() {
                writeln!(f, "**Notes:** {notes}")?;
            }
        }

        Ok(())
    }
}

pub struct NutritionAnalysisTool {
    api_key: String,
}

impl NutritionAnalysisTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Extract JSON from a natural language response that may contain extra text
    fn extract_json_from_response(&self, content: &str) -> Result<String, NutritionAnalysisError> {
        // Look for JSON object boundaries
        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                if end > start {
                    let json_str = &content[start..=end];
                    return Ok(json_str.to_string());
                }
            }
        }

        // If no JSON found, try to create a basic structure from natural language
        // This is a simple fallback - look for common patterns
        if content.to_lowercase().contains("ingredients")
            || content.to_lowercase().contains("grams")
        {
            // Try to parse natural language response into structured format
            return Err(NutritionAnalysisError::new(format!(
                "Could not extract JSON from response. The AI responded with natural language instead of JSON format. Response was: {content}"
            )));
        }

        Err(NutritionAnalysisError::new(format!(
            "No JSON object found in response: {content}"
        )))
    }

    /// Encode image file to base64 for OpenAI Vision API
    fn encode_image_to_base64(
        &self,
        image_path: &str,
    ) -> Result<(String, String), NutritionAnalysisError> {
        // Handle file:// URLs by stripping the protocol
        let clean_path = if let Some(stripped) = image_path.strip_prefix("file://") {
            stripped // Remove "file://" prefix
        } else {
            image_path
        };

        let path = Path::new(clean_path);

        // Check if file exists
        if !path.exists() {
            return Err(NutritionAnalysisError::new(format!(
                "Image file not found: {image_path}. Please check the file path and ensure the file exists. On macOS, try moving the image to a location like ~/Documents or ~/Pictures if it's on the Desktop."
            )));
        }

        // Show progress indicator
        print!("📷 Loading image...");
        io::stdout().flush().ok();

        // Read the file
        let image_data = std::fs::read(path).map_err(|e| {
            println!(" ❌");
            io::stdout().flush().ok();
            NutritionAnalysisError::new(format!("Failed to read image file: {e}. This might be due to macOS file access permissions. Try moving the image to ~/Documents or ~/Pictures, or grant terminal access to Desktop in System Preferences > Security & Privacy > Privacy > Files and Folders."))
        })?;

        print!(" ✅\n🔄 Encoding image...");
        io::stdout().flush().ok();

        // Determine MIME type based on file extension
        let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => {
                return Err(NutritionAnalysisError::new(
                    "Unsupported image format. Please use JPG, PNG, GIF, or WebP".to_string(),
                ));
            }
        };

        // Encode to base64
        let base64_image = general_purpose::STANDARD.encode(&image_data);

        println!(" ✅");
        io::stdout().flush().ok();

        Ok((base64_image, mime_type.to_string()))
    }

    /// Create the vision analysis request payload
    fn create_vision_request(&self, base64_image: &str, mime_type: &str) -> serde_json::Value {
        serde_json::json!({
            "model": "gpt-4o",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": r#"CRITICAL: You must respond with ONLY valid JSON. No additional text before or after.

First, verify this image contains food or meal items. If it does NOT contain food, respond with this exact JSON:
{
  "ingredients": [],
  "summary": "No food items detected in this image.",
  "notes": "This image does not appear to contain a meal or food items that can be analyzed for nutrition."
}

If the image DOES contain food/meal items, analyze them and identify all visible ingredients with their estimated portions in grams.

Your response must be EXACTLY this JSON format with no other text:

{
  "ingredients": [
    {
      "ingredient": "ingredient name",
      "amount_grams": estimated_grams_as_number,
      "confidence": "high/medium/low"
    }
  ],
  "summary": "Brief description of the meal",
  "notes": "Any important observations or uncertainties (optional)"
}

Guidelines:
- Be specific with ingredient names (e.g., "steel cut oats" not just "oats")
- Estimate portions based on typical serving sizes and visual cues
- Use confidence levels: high (very sure), medium (reasonable estimate), low (difficult to assess)
- Include all visible ingredients, even small amounts like seasonings if clearly visible
- For mixed dishes, break down individual components where possible
- If portion estimation is difficult, explain why in the notes

REMEMBER: Respond with ONLY the JSON object. No explanatory text."#
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:{};base64,{}", mime_type, base64_image),
                                "detail": "high"
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 1000,
            "temperature": 0.0
        })
    }
}

impl Tool for NutritionAnalysisTool {
    const NAME: &'static str = "analyze_meal_photo";

    type Error = NutritionAnalysisError;
    type Args = NutritionAnalysisArgs;
    type Output = NutritionAnalysisResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Analyze meal photos to identify ingredients and portions in grams. Use when user: mentions image file paths (.jpg, .png, etc.), asks to analyze meal/food photos, wants nutrition breakdown from images, or provides file paths like ~/Desktop/image.jpg or /Users/path/to/photo.png. Essential for Cronometer meal logging.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "image_path": {
                        "type": "string",
                        "description": "Full path to the meal image file (supports JPG, PNG, GIF, WebP). Extract this from the user's message."
                    }
                },
                "required": ["image_path"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate API key
        if self.api_key.is_empty() {
            return Err(NutritionAnalysisError::new(
                "OpenAI API key not configured. Run: vy config set llm_api_key",
            ));
        }

        // Encode the image
        let (base64_image, mime_type) = self.encode_image_to_base64(&args.image_path)?;

        print!("🧠 Analyzing meal with AI...");
        io::stdout().flush().ok();

        // Create the request payload
        let request_payload = self.create_vision_request(&base64_image, &mime_type);

        // Make the API request
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                println!(" ❌");
                io::stdout().flush().ok();
                NutritionAnalysisError::new(format!("Failed to send request to OpenAI: {e}. Check your internet connection and API key."))
            })?;

        if !response.status().is_success() {
            println!(" ❌");
            io::stdout().flush().ok();
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(NutritionAnalysisError::new(format!(
                "OpenAI API request failed with status {status}: {error_text}. This might be due to API quota limits or invalid API key."
            )));
        }

        print!(" ✅\n🔍 Processing results...");
        io::stdout().flush().ok();

        let response_text = response.text().await.map_err(|e| {
            println!(" ❌");
            io::stdout().flush().ok();
            NutritionAnalysisError::new(format!("Failed to read response: {e}"))
        })?;

        // Parse the OpenAI response
        let api_response: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| {
                println!(" ❌");
                io::stdout().flush().ok();
                NutritionAnalysisError::new(format!("Failed to parse API response: {e}"))
            })?;

        // Extract the content from the response
        let content = api_response
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| {
                println!(" ❌");
                io::stdout().flush().ok();
                NutritionAnalysisError::new(
                    "Invalid API response format - no content found in response",
                )
            })?;

        // Check if AI refused to analyze (common with non-meal images)
        if content.contains("I'm sorry, I can't assist") || content.contains("I cannot") {
            return Err(NutritionAnalysisError::new(
                "The AI couldn't analyze this image. This might be because:\n1. The image doesn't contain food/meal\n2. The image quality is too poor\n3. The image contains content that can't be analyzed\nPlease try with a clear photo of a meal or food items.".to_string()
            ));
        }

        // Clean up the response content - remove markdown formatting if present
        let cleaned_content = content
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        // Try to parse the JSON response from GPT-4V
        let nutrition_data: NutritionAnalysisResponse = match serde_json::from_str(cleaned_content)
        {
            Ok(data) => data,
            Err(_) => {
                // Fallback: try to extract JSON from natural language response
                let json_content = self.extract_json_from_response(cleaned_content)?;
                serde_json::from_str(&json_content).map_err(|e| {
                        println!(" ❌");
                        io::stdout().flush().ok();
                        NutritionAnalysisError::new(format!(
                            "Failed to parse nutrition analysis even after extraction: {e}. The AI may have responded with unexpected format. Cleaned response: {cleaned_content}"
                        ))
                    })?
            }
        };

        print!(" ✅\n\n");
        io::stdout().flush().ok();

        Ok(nutrition_data)
    }
}
