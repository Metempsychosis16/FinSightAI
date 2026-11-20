
import { useMemo, useState } from 'react'
import type { DatasetTransaction } from '../types/investigation'
import './TransactionExplorer.css'


interface TransactionExplorerProps {
  transactions: DatasetTransaction[]
}

type TransactionFilter =
  | 'ALL'
  | 'transfer'
  | 'deposit'
  | 'withdrawal'
  | 'payment'

type SortField = 'date' | 'amount'

function formatMoney(amountMinor: number, currency: string): string {
  try {
    const formatter = new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency,
    })
const fractionDigits =
  formatter.resolvedOptions().maximumFractionDigits ?? 2

return formatter.format(
  amountMinor / Math.pow(10, fractionDigits),
)



    

  } catch {
    return `${amountMinor} minor units (${currency})`
  }
}

function TransactionExplorer({
  transactions,
}: TransactionExplorerProps) {
  const [search, setSearch] = useState('')
  const [filter, setFilter] =
    useState<TransactionFilter>('ALL')
  const [sortField, setSortField] =
    useState<SortField>('date')
  const [descending, setDescending] = useState(true)
  const [selectedTransaction, setSelectedTransaction] =
    useState<DatasetTransaction | null>(null)

  const filteredTransactions = useMemo(() => {
    const query = search.trim().toLowerCase()

    const result = transactions.filter((transaction) => {
      const matchesSearch =
        !query ||
        transaction.external_reference.toLowerCase().includes(query) ||
        transaction.sender_account.toLowerCase().includes(query) ||
        transaction.recipient_account.toLowerCase().includes(query)

      const matchesFilter =
        filter === 'ALL' ||
        transaction.transaction_type.toLowerCase() === filter

      return matchesSearch && matchesFilter
    })

    result.sort((a, b) => {
      const comparison =
        sortField === 'amount'
          ? a.amount_minor - b.amount_minor
          : Date.parse(a.occurred_at) -
            Date.parse(b.occurred_at)

      return descending ? -comparison : comparison
    })

    return result
  }, [transactions, search, filter, sortField, descending])

  const currencyTotals = useMemo(() => {
    const totals = new Map<string, number>()

    for (const transaction of transactions) {
      totals.set(
        transaction.currency,
        (totals.get(transaction.currency) ?? 0) +
          transaction.amount_minor,
      )
    }

    return Array.from(totals.entries())
  }, [transactions])

  function toggleSort(field: SortField) {
    if (sortField === field) {
      setDescending((current) => !current)
    } else {
      setSortField(field)
      setDescending(true)
    }
  }

  return (
    <div className="transaction-explorer">
      <div className="transaction-summary">
        <article>
          <span>Total transactions</span>
          <strong>{transactions.length}</strong>
        </article>

        {currencyTotals.map(([currency, amount]) => (
          <article key={currency}>
            <span>Total transaction value · {currency}</span>
            <strong>{formatMoney(amount, currency)}</strong>
          </article>
        ))}
      </div>

      <div className="transaction-toolbar">
        <input
          type="search"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search reference or account..."
          aria-label="Search transactions"
        />

        <select
          value={filter}
          onChange={(event) =>
            setFilter(event.target.value as TransactionFilter)
          }
          aria-label="Filter transaction type"
        >
          <option value="ALL">All transaction types</option>
          <option value="transfer">Transfer</option>
          <option value="deposit">Deposit</option>
          <option value="withdrawal">Withdrawal</option>
          <option value="payment">Payment</option>
        </select>

        <button
          type="button"
          onClick={() => toggleSort('date')}
        >
          Sort by date {sortField === 'date' ? (descending ? '↓' : '↑') : ''}
        </button>

        <button
          type="button"
          onClick={() => toggleSort('amount')}
        >
          Sort by amount {sortField === 'amount' ? (descending ? '↓' : '↑') : ''}
        </button>
      </div>

      <p className="transaction-result-count" role="status">
        Showing {filteredTransactions.length} of {transactions.length} transactions
      </p>

      {filteredTransactions.length === 0 ? (
        <p className="transaction-empty">
          No transactions match your search or filter.
        </p>
      ) : (
        <div className="transaction-table-wrapper">
          <table className="transaction-table">
            <thead>
              <tr>
                <th>Reference</th>
                <th>Amount</th>
                <th>Type</th>
                <th>Sender</th>
                <th>Recipient</th>
                <th>Occurred at</th>
                <th>Details</th>
              </tr>
            </thead>

            <tbody>
              {filteredTransactions.map((transaction) => (
                <tr key={transaction.id}>
                  <td>{transaction.external_reference}</td>

                  <td>
                    {formatMoney(
                      transaction.amount_minor,
                      transaction.currency,
                    )}
                  </td>

                  <td>
                    <span className="transaction-type">
                      {transaction.transaction_type}
                    </span>
                  </td>

                  <td>{transaction.sender_account}</td>

                  <td>{transaction.recipient_account}</td>

                  <td>
                    {new Date(
                      transaction.occurred_at,
                    ).toLocaleString()}
                  </td>

                  <td>
                    <button
                      type="button"
                      className="transaction-detail-button"
                      onClick={() =>
                        setSelectedTransaction(transaction)
                      }
                    >
                      Inspect
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {selectedTransaction && (
        <section
          className="transaction-detail-panel"
          aria-labelledby="transaction-detail-title"
        >
          <div className="transaction-detail-heading">
            <h3 id="transaction-detail-title">
              Transaction details
            </h3>

            <button
              type="button"
              onClick={() => setSelectedTransaction(null)}
              aria-label="Close transaction details"
            >
              Close
            </button>
          </div>

          <dl>
            <dt>Transaction ID</dt>
            <dd>{selectedTransaction.id}</dd>

            <dt>External reference</dt>
            <dd>{selectedTransaction.external_reference}</dd>

            <dt>Amount</dt>
            <dd>
              {formatMoney(
                selectedTransaction.amount_minor,
                selectedTransaction.currency,
              )}
            </dd>

            <dt>Transaction type</dt>
            <dd>{selectedTransaction.transaction_type}</dd>

            <dt>Sender account</dt>
            <dd>{selectedTransaction.sender_account}</dd>

            <dt>Recipient account</dt>
            <dd>{selectedTransaction.recipient_account}</dd>

            <dt>Occurred at</dt>
            <dd>
              {new Date(
                selectedTransaction.occurred_at,
              ).toLocaleString()}
            </dd>

            <dt>Linked to case at</dt>
            <dd>
              {new Date(
                selectedTransaction.linked_at,
              ).toLocaleString()}
            </dd>
          </dl>
        </section>
      )}
    </div>
  )
}

export default TransactionExplorer

