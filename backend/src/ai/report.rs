use std::fmt::Write as _;

use super::output::{AnalysisFinding, StructuredInvestigationReport};

/// Converts a validated structured AI report into an investigator-readable
/// Markdown report.
///
/// This module formats existing analysis. It does not generate new findings,
/// verify factual claims, or make network/database requests.
pub struct InvestigationReportGenerator;

impl InvestigationReportGenerator {
    pub fn generate(report: &StructuredInvestigationReport) -> String {
        let mut output = String::new();

        writeln!(output, "# Financial Investigation Report\n").unwrap();

        writeln!(output, "**Case ID:** {}\n", report.case_id).unwrap();
        writeln!(
            output,
            "**Output schema version:** {}\n",
            escape_markdown(&report.schema_version)
        )
        .unwrap();

        writeln!(
            output,
            "> AI-assisted investigation report. This report is not a determination \
             that criminal activity occurred. A human investigator must review \
             the findings and underlying records.\n"
        )
        .unwrap();

        writeln!(output, "## 1. Executive Summary\n").unwrap();
        writeln!(output, "{}\n", escape_markdown(&report.executive_summary)).unwrap();

        write_findings(&mut output, "2. Verified Facts", &report.verified_facts);

        write_findings(
            &mut output,
            "3. Risk Engine Findings",
            &report.risk_engine_findings,
        );

        write_findings(
            &mut output,
            "4. Analytical Inferences",
            &report.analytical_inferences,
        );

        write_string_list(
            &mut output,
            "5. Missing Information",
            &report.missing_information,
        );

        write_string_list(
            &mut output,
            "6. Follow-up Questions",
            &report.follow_up_questions,
        );

        write_string_list(&mut output, "7. Overall Limitations", &report.limitations);

        writeln!(output, "---\n").unwrap();
        writeln!(
            output,
            "*End of AI-assisted report. Independent investigator review required.*"
        )
        .unwrap();

        output
    }
}

/// Writes one category of investigation findings.
fn write_findings(output: &mut String, heading: &str, findings: &[AnalysisFinding]) {
    writeln!(output, "## {heading}\n").unwrap();

    if findings.is_empty() {
        writeln!(output, "No findings supplied.\n").unwrap();
        return;
    }

    for (index, finding) in findings.iter().enumerate() {
        writeln!(output, "### Finding {}\n", index + 1).unwrap();

        writeln!(
            output,
            "**Description:** {}\n",
            escape_markdown(&finding.description)
        )
        .unwrap();

        writeln!(
            output,
            "**Rationale:** {}\n",
            escape_markdown(&finding.rationale)
        )
        .unwrap();

        writeln!(output, "**Limitations:**").unwrap();

        if finding.limitations.is_empty() {
            writeln!(output, "- None stated.\n").unwrap();
        } else {
            for limitation in &finding.limitations {
                writeln!(output, "- {}", escape_markdown(limitation)).unwrap();
            }
            writeln!(output).unwrap();
        }

        writeln!(
            output,
            "**Transaction IDs:** {}\n",
            format_ids(&finding.transaction_ids)
        )
        .unwrap();

        writeln!(
            output,
            "**Evidence IDs:** {}\n",
            format_ids(&finding.evidence_ids)
        )
        .unwrap();
    }
}

/// Writes a section containing ordinary text items.
fn write_string_list(output: &mut String, heading: &str, items: &[String]) {
    writeln!(output, "## {heading}\n").unwrap();

    if items.is_empty() {
        writeln!(output, "None supplied.\n").unwrap();
        return;
    }

    for item in items {
        writeln!(output, "- {}", escape_markdown(item)).unwrap();
    }

    writeln!(output).unwrap();
}

/// Formats numeric record IDs without treating them as Markdown input.
fn format_ids(ids: &[i64]) -> String {
    if ids.is_empty() {
        return "None supplied".to_owned();
    }

    ids.iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Escapes Markdown control characters in AI-generated text.
///
/// Newlines are converted to spaces so AI output cannot inject
/// additional headings or sections into the generated report.
fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '\n' | '\r' => escaped.push(' '),

            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!'
            | '|' | '>' | '<' => {
                escaped.push('\\');
                escaped.push(character);
            }

            other => escaped.push(other),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::InvestigationReportGenerator;
    use crate::ai::output::{
        AnalysisFinding, OUTPUT_SCHEMA_VERSION, StructuredInvestigationReport,
    };

    fn sample_report() -> StructuredInvestigationReport {
        StructuredInvestigationReport {
            schema_version: OUTPUT_SCHEMA_VERSION.to_owned(),
            case_id: 42,
            executive_summary: "Two transactions require further review.".to_owned(),

            verified_facts: vec![AnalysisFinding {
                description: "Transaction 10 is present in the dataset.".to_owned(),
                rationale: "The transaction record was supplied.".to_owned(),
                limitations: vec!["The transaction purpose is not established.".to_owned()],
                transaction_ids: vec![10],
                evidence_ids: vec![20],
            }],

            risk_engine_findings: vec![],
            analytical_inferences: vec![],

            missing_information: vec!["The purpose of the transaction is unknown.".to_owned()],

            follow_up_questions: vec!["Can the transaction purpose be verified?".to_owned()],

            limitations: vec!["Document contents were not fetched or parsed.".to_owned()],
        }
    }

    #[test]
    fn generates_report_with_all_expected_sections() {
        let report = sample_report();
        let markdown = InvestigationReportGenerator::generate(&report);

        assert!(markdown.contains("# Financial Investigation Report"));
        assert!(markdown.contains("Case ID:** 42"));
        assert!(markdown.contains("## 1. Executive Summary"));
        assert!(markdown.contains("## 2. Verified Facts"));
        assert!(markdown.contains("## 3. Risk Engine Findings"));
        assert!(markdown.contains("## 4. Analytical Inferences"));
        assert!(markdown.contains("## 5. Missing Information"));
        assert!(markdown.contains("## 6. Follow-up Questions"));
        assert!(markdown.contains("## 7. Overall Limitations"));
    }

    #[test]
    fn includes_finding_explanations_and_record_references() {
        let report = sample_report();
        let markdown = InvestigationReportGenerator::generate(&report);

        assert!(markdown.contains("The transaction record was supplied."));
        assert!(markdown.contains("The transaction purpose is not established."));
        assert!(markdown.contains("Transaction IDs:** 10"));
        assert!(markdown.contains("Evidence IDs:** 20"));
    }

    #[test]
    fn reports_empty_finding_categories() {
        let report = sample_report();
        let markdown = InvestigationReportGenerator::generate(&report);

        assert!(markdown.contains("No findings supplied."));
    }

    #[test]
    fn escapes_markdown_in_ai_generated_text() {
        let mut report = sample_report();

        report.executive_summary =
            "# Fake heading\n[malicious link](https://example.com)".to_owned();

        let markdown = InvestigationReportGenerator::generate(&report);

        assert!(!markdown.contains("\n# Fake heading"));
        assert!(markdown.contains("\\# Fake heading"));
        assert!(markdown.contains("\\[malicious link\\]"));
    }
}
