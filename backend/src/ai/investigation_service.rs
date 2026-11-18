use super::{
    evidence_package::InvestigationEvidencePackage,
    output::{OutputValidationError, StructuredInvestigationReport},
    prompt::{PromptBuildError, PromptBuilder},
    provider::{AiProviderError, LlmProvider},
};

#[derive(Debug)]
pub enum StructuredAnalysisError {
    Provider(AiProviderError),
    InvalidJson(String),
    Validation(OutputValidationError),
}

impl std::fmt::Display for StructuredAnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(error) => write!(f, "{error}"),
            Self::InvalidJson(error) => write!(f, "AI returned invalid JSON: {error}"),
            Self::Validation(error) => write!(f, "AI output validation failed: {error}"),
        }
    }
}

impl std::error::Error for StructuredAnalysisError {}

pub struct StructuredAnalysisResult {
    pub report: StructuredInvestigationReport,
    pub provider: String,
    pub model: String,
}

pub struct InvestigationAiService<P> {
    provider: P,
}

impl<P> InvestigationAiService<P>
where
    P: LlmProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub async fn analyze(
        &self,
        package: &InvestigationEvidencePackage,
    ) -> Result<super::provider::LlmResponse, AiProviderError> {
        let request = PromptBuilder::build(package)
            .map_err(|error: PromptBuildError| AiProviderError::RequestFailed(error.to_string()))?;

        self.provider.generate(&request).await
    }

    pub async fn analyze_structured_with_metadata(
        &self,
        package: &InvestigationEvidencePackage,
    ) -> Result<StructuredAnalysisResult, StructuredAnalysisError> {
        let response = self
            .analyze(package)
            .await
            .map_err(StructuredAnalysisError::Provider)?;

        let report: StructuredInvestigationReport = serde_json::from_str(&response.content)
            .map_err(|error| StructuredAnalysisError::InvalidJson(error.to_string()))?;

        report
            .validate_against(package)
            .map_err(StructuredAnalysisError::Validation)?;

        Ok(StructuredAnalysisResult {
            report,
            provider: response.provider,
            model: response.model,
        })
    }

    pub async fn analyze_structured(
        &self,
        package: &InvestigationEvidencePackage,
    ) -> Result<StructuredInvestigationReport, StructuredAnalysisError> {
        let result = self.analyze_structured_with_metadata(package).await?;

        Ok(result.report)
    }
}
