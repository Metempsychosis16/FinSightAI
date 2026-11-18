use std::{error::Error, fmt, future::Future, pin::Pin};

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// A future returned by an LLM provider.
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Provider-independent request sent to an LLM.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system_instruction: String,
    pub user_prompt: String,
}

/// Provider-independent response received from an LLM.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub provider: String,
    pub model: String,
    pub content: String,
}

/// Errors that can occur while communicating with an LLM provider.
#[derive(Debug)]
pub enum AiProviderError {
    Configuration(String),
    Authentication(String),
    RateLimited(String),
    RequestFailed(String),
    InvalidResponse(String),
}

#[derive(Clone)]
pub struct RequestyProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl fmt::Display for AiProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(f, "AI configuration error: {message}"),
            Self::Authentication(message) => write!(f, "AI authentication error: {message}"),
            Self::RateLimited(message) => write!(f, "AI rate limit reached: {message}"),
            Self::RequestFailed(message) => write!(f, "AI request failed: {message}"),
            Self::InvalidResponse(message) => write!(f, "Invalid AI response: {message}"),
        }
    }
}

impl Error for AiProviderError {}

pub trait LlmProvider: Send + Sync {
    fn generate<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> ProviderFuture<'a, Result<LlmResponse, AiProviderError>>;
}

impl RequestyProvider {
    pub fn new(api_key: String, model: String, base_url: String) -> Result<Self, AiProviderError> {
        if api_key.trim().is_empty() {
            return Err(AiProviderError::Configuration(
                "REQUESTY_API_KEY is empty".to_owned(),
            ));
        }

        if model.trim().is_empty() {
            return Err(AiProviderError::Configuration(
                "REQUESTY_MODEL is empty".to_owned(),
            ));
        }

        let base_url = base_url.trim_end_matches('/').to_owned();

        if !base_url.starts_with("https://") {
            return Err(AiProviderError::Configuration(
                "Requesty base URL must use HTTPS".to_owned(),
            ));
        }

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
            base_url,
        })
    }

    /// Read provider configuration from environment variables.
    pub fn from_env() -> Result<Self, AiProviderError> {
        let api_key = std::env::var("REQUESTY_API_KEY").map_err(|_| {
            AiProviderError::Configuration("REQUESTY_API_KEY is not set".to_owned())
        })?;

        let model = std::env::var("REQUESTY_MODEL")
            .map_err(|_| AiProviderError::Configuration("REQUESTY_MODEL is not set".to_owned()))?;

        let base_url = std::env::var("REQUESTY_BASE_URL")
            .unwrap_or_else(|_| "https://router.requesty.ai/v1".to_owned());

        Self::new(api_key, model, base_url)
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: [ChatMessage<'a>; 2],
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

impl LlmProvider for RequestyProvider {
    fn generate<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> ProviderFuture<'a, Result<LlmResponse, AiProviderError>> {
        Box::pin(async move {
            let endpoint = format!("{}/chat/completions", self.base_url);

            let body = ChatCompletionRequest {
                model: &self.model,
                messages: [
                    ChatMessage {
                        role: "system",
                        content: &request.system_instruction,
                    },
                    ChatMessage {
                        role: "user",
                        content: &request.user_prompt,
                    },
                ],
            };

            let response = self
                .client
                .post(&endpoint)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .header("X-Title", "FinSight AI")
                .json(&body)
                .send()
                .await
                .map_err(|error| AiProviderError::RequestFailed(error.to_string()))?;

            let status = response.status();

            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return Err(AiProviderError::Authentication(format!(
                    "Requesty returned HTTP {status}"
                )));
            }

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(AiProviderError::RateLimited(
                    "Requesty returned HTTP 429".to_owned(),
                ));
            }

            if !status.is_success() {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read error response".to_owned());

                return Err(AiProviderError::RequestFailed(format!(
                    "HTTP {status}: {error_body}"
                )));
            }

            let result: ChatCompletionResponse = response
                .json()
                .await
                .map_err(|error| AiProviderError::InvalidResponse(error.to_string()))?;

            let content = result
                .choices
                .into_iter()
                .next()
                .and_then(|choice| choice.message.content)
                .filter(|content| !content.trim().is_empty())
                .ok_or_else(|| {
                    AiProviderError::InvalidResponse("No assistant content in response".to_owned())
                })?;

            Ok(LlmResponse {
                provider: "requesty".to_owned(),
                model: result.model.unwrap_or_else(|| self.model.clone()),
                content,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AiProviderError, RequestyProvider};

    #[test]
    fn provider_rejects_empty_api_key() {
        let result = RequestyProvider::new(
            String::new(),
            "some/free-model".to_owned(),
            "https://router.requesty.ai/v1".to_owned(),
        );

        assert!(matches!(result, Err(AiProviderError::Configuration(_))));
    }

    #[test]
    fn provider_rejects_empty_model() {
        let result = RequestyProvider::new(
            "test-key".to_owned(),
            String::new(),
            "https://router.requesty.ai/v1".to_owned(),
        );

        assert!(matches!(result, Err(AiProviderError::Configuration(_))));
    }

    #[test]
    fn provider_rejects_non_https_base_url() {
        let result = RequestyProvider::new(
            "test-key".to_owned(),
            "some/free-model".to_owned(),
            "http://router.requesty.ai/v1".to_owned(),
        );

        assert!(matches!(result, Err(AiProviderError::Configuration(_))));
    }

    #[test]
    fn provider_accepts_valid_configuration() {
        let result = RequestyProvider::new(
            "test-key".to_owned(),
            "some/free-model".to_owned(),
            "https://router.requesty.ai/v1".to_owned(),
        );

        assert!(result.is_ok());
    }
}
