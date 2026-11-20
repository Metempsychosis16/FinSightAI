
import { useEffect, useMemo, useState } from 'react'
import { clearAccessToken } from '../../auth/services/tokenStorage'
import {
  getCases,
  getHighRiskCases,
} from '../services/dashboardService'
import type { Case } from '../../../shared/types/case'
import './DashboardPage.css'

interface DashboardPageProps {
  onLogout: () => void
  onOpenCase: (id: number) => void
}

type CaseView = 'ALL' | 'HIGH_RISK'
type StatusFilter = 'ALL' | Case['status']
type SortOption = 'NEWEST' | 'OLDEST' | 'HIGHEST_RISK' | 'LOWEST_RISK'

const PAGE_SIZE = 10

function DashboardPage({ onLogout, onOpenCase }: DashboardPageProps) {
  const [cases, setCases] = useState<Case[]>([])
  const [highRiskCases, setHighRiskCases] = useState<Case[]>([])

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [highRiskError, setHighRiskError] = useState('')

  const [caseView, setCaseView] = useState<CaseView>('ALL')
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('ALL')
  const [sortOption, setSortOption] = useState<SortOption>('NEWEST')
  const [currentPage, setCurrentPage] = useState(1)

  useEffect(() => {
    let isActive = true

    async function loadDashboard() {
      const [allCasesResult, highRiskResult] = await Promise.allSettled([
        getCases(),
        getHighRiskCases(),
      ])

      if (!isActive) return

      if (allCasesResult.status === 'fulfilled') {
        setCases(allCasesResult.value)
        setError('')
      } else {
        setError(
          allCasesResult.reason instanceof Error
            ? allCasesResult.reason.message
            : 'Unable to load cases.',
        )
      }

      if (highRiskResult.status === 'fulfilled') {
        setHighRiskCases(highRiskResult.value)
        setHighRiskError('')
      } else {
        setHighRiskCases([])
        setHighRiskError(
          highRiskResult.reason instanceof Error
            ? highRiskResult.reason.message
            : 'Unable to load high-risk cases.',
        )
      }

      setIsLoading(false)
    }

    void loadDashboard()

    return () => {
      isActive = false
    }
  }, [])

  function handleLogout() {
    clearAccessToken()
    onLogout()
  }

  // Reset pagination whenever the user changes a filter.
  function updateSearch(value: string) {
    setSearchTerm(value)
    setCurrentPage(1)
  }

  function updateView(value: CaseView) {
    setCaseView(value)
    setCurrentPage(1)
  }

  function updateStatus(value: StatusFilter) {
    setStatusFilter(value)
    setCurrentPage(1)
  }

  function updateSort(value: SortOption) {
    setSortOption(value)
    setCurrentPage(1)
  }

  const openCases = cases.filter((item) => item.status === 'OPEN').length

  const underReviewCases = cases.filter(
    (item) => item.status === 'UNDER_REVIEW',
  ).length

  const escalatedCases = cases.filter(
    (item) => item.status === 'ESCALATED',
  ).length

  const filteredCases = useMemo(() => {
    const sourceCases = caseView === 'HIGH_RISK' ? highRiskCases : cases

    const normalizedSearch = searchTerm.trim().replace(/^#/, '')

    const result = sourceCases.filter((item) => {
      const matchesCaseNumber =
        normalizedSearch === '' ||
        String(item.id).includes(normalizedSearch)

      const matchesStatus =
        statusFilter === 'ALL' || item.status === statusFilter

      return matchesCaseNumber && matchesStatus
    })

    return [...result].sort((a, b) => {
      switch (sortOption) {
        case 'OLDEST':
          return (
            new Date(a.created_at).getTime() -
            new Date(b.created_at).getTime()
          )

        case 'HIGHEST_RISK':
          return b.risk_score - a.risk_score

        case 'LOWEST_RISK':
          return a.risk_score - b.risk_score

        case 'NEWEST':
        default:
          return (
            new Date(b.created_at).getTime() -
            new Date(a.created_at).getTime()
          )
      }
    })
  }, [
    cases,
    highRiskCases,
    caseView,
    searchTerm,
    statusFilter,
    sortOption,
  ])

  const totalPages = Math.max(
    1,
    Math.ceil(filteredCases.length / PAGE_SIZE),
  )

  const safePage = Math.min(currentPage, totalPages)

  const paginatedCases = filteredCases.slice(
    (safePage - 1) * PAGE_SIZE,
    safePage * PAGE_SIZE,
  )

  const firstVisibleCase =
    filteredCases.length === 0 ? 0 : (safePage - 1) * PAGE_SIZE + 1

  const lastVisibleCase = Math.min(
    safePage * PAGE_SIZE,
    filteredCases.length,
  )

  return (
    <main className="dashboard-page">
      <header className="dashboard-header">
        <div className="dashboard-brand">
          <div className="dashboard-brand-mark">FS</div>
          <div>
            <strong>FinSight AI</strong>
            <span>Investigation Platform</span>
          </div>
        </div>

        <button
          className="logout-button"
          type="button"
          onClick={handleLogout}
        >
          Sign out
        </button>
      </header>

      <section className="dashboard-content">
        <div className="dashboard-title">
          <div>
            <p className="dashboard-eyebrow">OVERVIEW</p>
            <h1>Investigation Dashboard</h1>
            <p>Monitor cases and financial crime risk indicators.</p>
          </div>
        </div>

        {isLoading && (
          <p className="dashboard-feedback" role="status">
            Loading investigation data...
          </p>
        )}

        {!isLoading && error && (
          <div className="dashboard-error" role="alert">
            <p>{error}</p>
            <button
              type="button"
              onClick={() => window.location.reload()}
            >
              Reload dashboard
            </button>
          </div>
        )}

        {!isLoading && !error && (
          <>
            <section className="dashboard-stats" aria-label="Case statistics">
              <article className="stat-card">
                <span>Total cases</span>
                <strong>{cases.length}</strong>
                <small>All accessible cases</small>
              </article>

              <article className="stat-card">
                <span>Open cases</span>
                <strong>{openCases}</strong>
                <small>Awaiting investigation</small>
              </article>

              <article className="stat-card">
                <span>Under review</span>
                <strong>{underReviewCases}</strong>
                <small>Currently being reviewed</small>
              </article>

              <article className="stat-card">
                <span>Escalated cases</span>
                <strong>{escalatedCases}</strong>
                <small>Require further attention</small>
              </article>
            </section>

            <section className="dashboard-section">
              <div className="section-heading">
                <div>
                  <h2>Cases</h2>
                  <p>Browse and filter your accessible investigations.</p>
                </div>
                <span className="risk-count">
                  {filteredCases.length} results
                </span>
              </div>

              <div className="case-view-toggle" aria-label="Case view">
                <button
                  type="button"
                  className={caseView === 'ALL' ? 'active' : ''}
                  aria-pressed={caseView === 'ALL'}
                  onClick={() => updateView('ALL')}
                >
                  All cases ({cases.length})
                </button>

                <button
                  type="button"
                  className={caseView === 'HIGH_RISK' ? 'active' : ''}
                  aria-pressed={caseView === 'HIGH_RISK'}
                  onClick={() => updateView('HIGH_RISK')}
                >
                  High risk ({highRiskCases.length})
                </button>
              </div>

              {caseView === 'HIGH_RISK' && highRiskError && (
                <p className="dashboard-error" role="alert">
                  {highRiskError}
                </p>
              )}

              <div className="case-filters">
                <label className="case-search">
                  <span>Search by case number</span>
                  <input
                    type="search"
                    value={searchTerm}
                    onChange={(event) => updateSearch(event.target.value)}
                    placeholder="Enter case number, e.g. 102"
                  />
                </label>

                <label className="case-filter">
                  <span>Status</span>
                  <select
                    value={statusFilter}
                    onChange={(event) =>
                      updateStatus(event.target.value as StatusFilter)
                    }
                  >
                    <option value="ALL">All statuses</option>
                    <option value="OPEN">Open</option>
                    <option value="UNDER_REVIEW">Under review</option>
                    <option value="ESCALATED">Escalated</option>
                    <option value="CLOSED">Closed</option>
                    <option value="ARCHIVED">Archived</option>
                  </select>
                </label>

                <label className="case-filter">
                  <span>Sort by</span>
                  <select
                    value={sortOption}
                    onChange={(event) =>
                      updateSort(event.target.value as SortOption)
                    }
                  >
                    <option value="NEWEST">Newest first</option>
                    <option value="OLDEST">Oldest first</option>
                    <option value="HIGHEST_RISK">Highest risk</option>
                    <option value="LOWEST_RISK">Lowest risk</option>
                  </select>
                </label>

                <button
                  type="button"
                  className="clear-filters-button"
                  onClick={() => {
                    setSearchTerm('')
                    setStatusFilter('ALL')
                    setSortOption('NEWEST')
                    setCurrentPage(1)
                  }}
                >
                  Clear filters
                </button>
              </div>

              {filteredCases.length === 0 ? (
                <p className="empty-state">
                  No cases match your search or filters.
                </p>
              ) : (
                <>
                  <div className="case-table-wrapper">
                    <table className="case-table">
                      <thead>
                        <tr>
                          <th>Case number</th>
                          <th>Title</th>
                          <th>Risk score</th>
                          <th>Status</th>
                          <th>Created</th>
                          <th>Action</th>
                        </tr>
                      </thead>

                      <tbody>
                        {paginatedCases.map((item) => (
                          <tr key={item.id}>
                            <td>#{item.id}</td>

<td>
  <button
    type="button"
    className="case-open-link"
    onClick={() => onOpenCase(item.id)}
    aria-label={`Open investigation for case ${item.id}: ${item.title}`}
  >
    {item.title}
  </button>
</td>



                            <td>
                              <span className="risk-score">
                                {item.risk_score}/100
                              </span>
                            </td>

                            <td>
                              <span
                                className={`status status-${item.status.toLowerCase()}`}
                              >
                                {item.status.replaceAll('_', ' ')}
                              </span>
                            </td>

                            <td>
                              {new Date(item.created_at).toLocaleDateString()}
                            </td>

                            <td>
                              <button
                                type="button"
                                className="case-open-link"
                                onClick={() => onOpenCase(item.id)}
                              >
                                Open case
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  <div className="case-pagination">
                    <p>
                      Showing {firstVisibleCase}–{lastVisibleCase} of{' '}
                      {filteredCases.length} cases
                    </p>

                    <div className="pagination-controls">
                      <button
                        type="button"
                        disabled={safePage === 1}
                        onClick={() => setCurrentPage(safePage - 1)}
                      >
                        Previous
                      </button>

                      <span>
                        Page {safePage} of {totalPages}
                      </span>

                      <button
                        type="button"
                        disabled={safePage === totalPages}
                        onClick={() => setCurrentPage(safePage + 1)}
                      >
                        Next
                      </button>
                    </div>
                  </div>
                </>
              )}
            </section>

            <p className="dashboard-note">
              Dashboard statistics are calculated from all cases returned by
              the authenticated backend. High-risk cases are supplied by the
              backend high-risk endpoint.
            </p>
          </>
        )}
      </section>
    </main>
  )
}

export default DashboardPage

