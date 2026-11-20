import { useEffect, useState } from 'react'
import type { Case } from '../../../shared/types/case'
import {
  getCaseById,
  getInvestigationDataset,
  getAIReports,
} from '../services/investigationService'

import TransactionExplorer from '../components/TransactionExplorer'

import TransactionGraphExplorer from '../components/TransactionGraphExplorer'

import {
  getTransactionGraph,
} from '../services/investigationService'

import type {
  TransactionGraph,
} from '../types/transactionGraph'

import type {
  InvestigationDataset,
  AIInvestigationReport,
} from '../types/investigation'

import './InvestigationPage.css'

interface InvestigationPageProps {
  caseId: number
  onBack: () => void
}

function InvestigationPage({
  caseId,
  onBack,
}: InvestigationPageProps) {
  const [caseData, setCaseData] = useState<Case | null>(null)

  const [dataset, setDataset] =
    useState<InvestigationDataset | null>(null)

  const [reports, setReports] = useState<AIInvestigationReport[]>([])
const [graph, setGraph] =
  useState<TransactionGraph | null>(null)

const [graphError, setGraphError] = useState('')


  const [caseError, setCaseError] = useState('')
  const [datasetError, setDatasetError] = useState('')
  const [reportsError, setReportsError] = useState('')

  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    let active = true

    async function loadInvestigation() {
      setIsLoading(true)

      const caseResult = await getCaseById(caseId).then(
        (data) => ({ success: true as const, data }),
        (error: unknown) => ({ success: false as const, error }),
      )

      if (!active) return

      if (!caseResult.success) {
        setCaseError(
          caseResult.error instanceof Error
            ? caseResult.error.message
            : 'Unable to retrieve case.',
        )
        setIsLoading(false)
        return
      }

      setCaseData(caseResult.data)

      // These endpoints have additional backend role restrictions.
      // Fetch them independently so one failure does not hide the other.
      
const [
  datasetResult,
  reportsResult,
  graphResult,
] = await Promise.allSettled([
  getInvestigationDataset(caseId),
  getAIReports(caseId),
  getTransactionGraph(caseId),
])

      
      if (!active) return

      if (datasetResult.status === 'fulfilled') {
        setDataset(datasetResult.value)
        setDatasetError('')
      } else {
        setDatasetError(
          datasetResult.reason instanceof Error
            ? datasetResult.reason.message
            : 'Unable to retrieve investigation dataset.',
        )
      }

      if (reportsResult.status === 'fulfilled') {
        setReports(reportsResult.value)
        setReportsError('')
      } else {
        setReportsError(
          reportsResult.reason instanceof Error
            ? reportsResult.reason.message
            : 'Unable to retrieve AI reports.',
        )
      }

if (graphResult.status === 'fulfilled') {
  setGraph(graphResult.value)
  setGraphError('')
} else {
  setGraph(null)

  setGraphError(
    graphResult.reason instanceof Error
      ? graphResult.reason.message
      : 'Unable to retrieve transaction graph.',
  )
}



      setIsLoading(false)
    }

    void loadInvestigation()

    return () => {
      active = false
    }
  }, [caseId])

  if (isLoading) {
    return (
      <main className="investigation-page">
        <p className="investigation-loading" role="status">
          Loading case investigation...
        </p>
      </main>
    )
  }

  if (caseError || !caseData) {
    return (
      <main className="investigation-page">
        <button className="back-button" onClick={onBack}>
          ← Back to dashboard
        </button>

        <div className="investigation-error" role="alert">
          {caseError || 'Case not found.'}
        </div>
      </main>
    )
  }

  return (
    <main className="investigation-page">
      <header className="investigation-header">
        <div className="investigation-brand">
          <div className="investigation-brand-mark">FS</div>
          <div>
            <strong>FinSight AI</strong>
            <span>Case Investigation Workspace</span>
          </div>
        </div>

        <button className="back-button" onClick={onBack}>
          ← Back to dashboard
        </button>
      </header>

      <div className="investigation-content">
        {/* CASE OVERVIEW */}

        <section className="investigation-case">
          <div>
            <p className="investigation-eyebrow">
              CASE INVESTIGATION
            </p>

            <h1>{caseData.title}</h1>

            <p className="case-reference">
              Case #{caseData.id}
            </p>
          </div>

          <div className="case-overview-metrics">
            <div className="case-metric">
              <span>Risk score</span>
              <strong>{caseData.risk_score}/100</strong>
            </div>

            <div className="case-metric">
              <span>Status</span>
              <strong className="case-status">
                {caseData.status.replaceAll('_', ' ')}
              </strong>
            </div>
          </div>
        </section>

        {/* INVESTIGATION SUMMARY */}

        <section className="investigation-summary">
          <article>
            <span>Linked transactions</span>
            <strong>
              {dataset ? dataset.transaction_count : '—'}
            </strong>
          </article>

          <article>
            <span>Evidence records</span>
            <strong>
              {dataset ? dataset.evidence_count : '—'}
            </strong>
          </article>

          <article>
            <span>AI reports</span>
            <strong>{reports.length}</strong>
          </article>
        </section>

        {/* INVESTIGATOR */}

        <section className="investigation-panel">
          <div className="panel-heading">
            <h2>Assigned investigator</h2>
          </div>

          {datasetError ? (
            <p className="restricted-message" role="status">
              Investigator information is unavailable. You may not
              have permission to access the investigation dataset.
            </p>
          ) : dataset?.investigator ? (
            <div className="investigator-details">
              <strong>{dataset.investigator.name}</strong>
              <span>{dataset.investigator.department}</span>
              <span>
                {dataset.investigator.years_experience} years of
                experience
              </span>
            </div>
          ) : (
            <p className="empty-message">
              No investigator assigned or no dataset available.
            </p>
          )}
        </section>


{/* TRANSACTIONS */}

<section className="investigation-panel">
  <div className="panel-heading">
    <div>
      <h2>Transaction Explorer</h2>
      <p>
        Search, filter, sort and inspect financial transactions
        linked to this investigation.
      </p>
    </div>
  </div>

  {datasetError ? (
    <p className="restricted-message" role="status">
      Transaction information requires access to the
      investigation dataset.
    </p>
  ) : dataset ? (
    <TransactionExplorer
      transactions={dataset.transactions}
    />
  ) : (
    <p className="empty-message">
      Investigation dataset unavailable.
    </p>
  )}

</section>


{/* TRANSACTION GRAPH */}

<section className="investigation-panel">
  <div className="panel-heading">
    <div>
      <h2>Transaction Network Graph</h2>
      <p>
        Visualize account relationships and transaction
        direction within this investigation.
      </p>
    </div>
  </div>

  {graphError ? (
    <p className="restricted-message" role="status">
      Transaction graph is unavailable.
      Access may be restricted to authorized investigation roles.
    </p>
  ) : graph ? (
    <TransactionGraphExplorer graph={graph} />
  ) : (
    <p className="empty-message">
      Transaction graph unavailable.
    </p>
  )}
</section>






                {/* EVIDENCE */}

        <section className="investigation-panel">
          <div className="panel-heading">
            <div>
              <h2>Investigation evidence</h2>
              <p>Evidence records associated with this case.</p>
            </div>
          </div>

          {datasetError ? (
            <p className="restricted-message" role="status">
              Evidence information requires access to the
              investigation dataset.
            </p>
          ) : !dataset ? (
            <p className="empty-message">
              Evidence information unavailable.
            </p>
          ) : dataset.evidence.length === 0 ? (
            <p className="empty-message">
              No evidence records have been added.
            </p>
          ) : (
            <div className="evidence-list">
              {dataset.evidence.map((item) => (
                <article className="evidence-card" key={item.id}>
                  <div>
                    <span className="evidence-type">
                      {item.evidence_type.replaceAll('_', ' ')}
                    </span>

                    <h3>{item.title}</h3>

                    <p>{item.description}</p>

                    <small>
                      Source: {item.source_reference}
                    </small>
                  </div>

                  <div className="evidence-metadata">
                    <span>Evidence #{item.id}</span>

                    <span>
                      {new Date(
                        item.created_at,
                      ).toLocaleDateString()}
                    </span>

                    {item.sha256_hash && (
                      <span className="hash-value">
                        SHA-256: {item.sha256_hash}
                      </span>
                    )}
                  </div>
                </article>
              ))}
            </div>
          )}
        </section>

        {/* AI REPORTS */}

        <section className="investigation-panel">
          <div className="panel-heading">
            <div>
              <h2>AI investigation reports</h2>
              <p>Previously generated and stored reports.</p>
            </div>
          </div>

          {reportsError ? (
            <p className="restricted-message" role="status">
              AI reports are restricted to authorized investigation
              roles or could not be retrieved.
            </p>
          ) : reports.length === 0 ? (
            <p className="empty-message">
              No AI investigation reports are available.
            </p>
          ) : (
            <div className="report-list">
              {reports.map((report) => (
                <article className="report-card" key={report.id}>
                  <div className="report-header">
                    <div>
                      <h3>Investigation report #{report.id}</h3>

                      <p>
                        {report.provider} · {report.model}
                      </p>
                    </div>

                    <span className="report-date">
                      {new Date(
                        report.created_at,
                      ).toLocaleString()}
                    </span>
                  </div>

                  <div className="report-version">
                    Prompt: {report.prompt_version}
                    {' · '}
                    Schema: {report.schema_version}
                  </div>

                  <details>
                    <summary>Read full investigation report</summary>

                    <pre className="markdown-report">
                      {report.markdown_report}
                    </pre>
                  </details>

                  <details>
                    <summary>Inspect structured report (JSON)</summary>

                    <pre className="structured-report">
                      {JSON.stringify(
                        report.structured_report,
                        null,
                        2,
                      )}
                    </pre>
                  </details>
                </article>
              ))}
            </div>
          )}
        </section>

        <footer className="investigation-footer">
          FinSight AI · Investigation workspace · Case #{caseData.id}
        </footer>
      </div>
    </main>
  )
}

export default InvestigationPage
