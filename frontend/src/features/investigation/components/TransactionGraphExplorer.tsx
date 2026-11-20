
import { useMemo, useState } from 'react'
import type {
  AccountNode,
  AccountRelationship,
  TransactionEdge,
  TransactionGraph,
} from '../types/transactionGraph'
import './TransactionGraphExplorer.css'

interface TransactionGraphExplorerProps {
  graph: TransactionGraph
}

interface NodePosition {
  x: number
  y: number
}

const WIDTH = 760
const HEIGHT = 480
const CENTER_X = WIDTH / 2
const CENTER_Y = HEIGHT / 2

function TransactionGraphExplorer({
  graph,
}: TransactionGraphExplorerProps) {
  const [selectedAccount, setSelectedAccount] =
    useState<string | null>(null)

  const [selectedRelationship, setSelectedRelationship] =
    useState<AccountRelationship | null>(null)

  const positions = useMemo(() => {
    const result = new Map<string, NodePosition>()
    const count = graph.nodes.length

    if (count === 0) return result

    const radius = Math.min(
      WIDTH * 0.34,
      HEIGHT * 0.36,
    )

    graph.nodes.forEach((node: AccountNode, index) => {
      const angle =
        (2 * Math.PI * index) / count - Math.PI / 2

      result.set(node.account_id, {
        x: CENTER_X + radius * Math.cos(angle),
        y: CENTER_Y + radius * Math.sin(angle),
      })
    })

    return result
  }, [graph.nodes])

  const accountMetrics = useMemo(() => {
    if (!selectedAccount) return null

    const incoming = graph.edges.filter(
      (edge: TransactionEdge) =>
        edge.to_account === selectedAccount,
    )

    const outgoing = graph.edges.filter(
      (edge: TransactionEdge) =>
        edge.from_account === selectedAccount,
    )

    const senders = new Set(
      incoming.map((edge) => edge.from_account),
    )

    const recipients = new Set(
      outgoing.map((edge) => edge.to_account),
    )

    const connections = new Set([
      ...senders,
      ...recipients,
    ])

    return {
      incomingCount: incoming.length,
      outgoingCount: outgoing.length,
      uniqueSenders: senders.size,
      uniqueRecipients: recipients.size,
      totalConnections: connections.size,
    }
  }, [graph.edges, selectedAccount])

  const selectedTransactions = useMemo(() => {
    if (!selectedRelationship) return []

    const ids = new Set(
      selectedRelationship.transaction_ids,
    )

    return graph.edges.filter((edge) =>
      ids.has(edge.transaction_id),
    )
  }, [graph.edges, selectedRelationship])

  function selectAccount(accountId: string) {
    setSelectedAccount(accountId)
    setSelectedRelationship(null)
  }

  function selectRelationship(
    relationship: AccountRelationship,
  ) {
    setSelectedRelationship(relationship)
    setSelectedAccount(null)
  }

  if (graph.nodes.length === 0) {
    return (
      <div className="graph-empty">
        No account relationships are available for this case.
      </div>
    )
  }

  return (
    <div className="transaction-graph-explorer">
      <div className="graph-summary">
        <article>
          <span>Accounts</span>
          <strong>{graph.nodes.length}</strong>
        </article>

        <article>
          <span>Transactions</span>
          <strong>{graph.edges.length}</strong>
        </article>

        <article>
          <span>Relationships</span>
          <strong>{graph.relationships.length}</strong>
        </article>
      </div>

      <div className="graph-legend">
        <span>
          <i className="graph-node-legend" />
          Account
        </span>

        <span>
          <i className="graph-edge-legend" />
          Transaction direction
        </span>
      </div>

      <div className="graph-canvas">
        <svg
          viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
          role="group"
          aria-label="Directed transaction network. Select an account or relationship to inspect it."
        >
          <defs>
            <marker
              id="transaction-arrow"
              markerWidth="8"
              markerHeight="8"
              refX="7"
              refY="3"
              orient="auto"
            >
              <path
                d="M0 0 L7 3 L0 6"
                fill="none"
                stroke="#60a5fa"
                strokeWidth="1.5"
              />
            </marker>
          </defs>

          {graph.relationships.map((relationship, index) => {
            const from = positions.get(
              relationship.from_account,
            )

            const to = positions.get(
              relationship.to_account,
            )

            if (!from || !to) return null

            const dx = to.x - from.x
            const dy = to.y - from.y
            const length = Math.sqrt(dx * dx + dy * dy)

            if (length === 0) return null

            const unitX = dx / length
            const unitY = dy / length

            const startX = from.x + unitX * 26
            const startY = from.y + unitY * 26
            const endX = to.x - unitX * 29
            const endY = to.y - unitY * 29

            const midX = (startX + endX) / 2
            const midY = (startY + endY) / 2

            const isSelected =
              selectedRelationship === relationship

            return (
              <g
                key={`${relationship.from_account}-${relationship.to_account}-${index}`}
                className="graph-relationship"
                onClick={() =>
                  selectRelationship(relationship)
                }
                role="button"
                tabIndex={0}
                aria-label={`${relationship.from_account} to ${relationship.to_account}, ${relationship.transaction_count} transactions`}
                onKeyDown={(event) => {
                  if (
                    event.key === 'Enter' ||
                    event.key === ' '
                  ) {
                    event.preventDefault()
                    selectRelationship(relationship)
                  }
                }}
              >
                <line
                  x1={startX}
                  y1={startY}
                  x2={endX}
                  y2={endY}
                  stroke={isSelected ? '#f59e0b' : '#60a5fa'}
                  strokeWidth={isSelected ? 4 : 2}
                  markerEnd="url(#transaction-arrow)"
                />

                <rect
                  x={midX - 17}
                  y={midY - 12}
                  width="34"
                  height="24"
                  rx="7"
                  fill="#172d49"
                  stroke="#405b7d"
                />

                <text
                  x={midX}
                  y={midY + 4}
                  textAnchor="middle"
                  fill="#e2e8f0"
                  fontSize="11"
                >
                  {relationship.transaction_count}
                </text>
              </g>
            )
          })}

          {graph.nodes.map((node: AccountNode) => {
            const position = positions.get(node.account_id)

            if (!position) return null

            const isSelected =
              selectedAccount === node.account_id

            return (
              <g
                key={node.account_id}
                className="graph-account"
                transform={`translate(${position.x}, ${position.y})`}
                onClick={() =>
                  selectAccount(node.account_id)
                }
                role="button"
                tabIndex={0}
                aria-label={`Inspect account ${node.account_id}`}
                onKeyDown={(event) => {
                  if (
                    event.key === 'Enter' ||
                    event.key === ' '
                  ) {
                    event.preventDefault()
                    selectAccount(node.account_id)
                  }
                }}
              >
                <circle
                  r="27"
                  fill={isSelected ? '#713f12' : '#172d49'}
                  stroke={isSelected ? '#f59e0b' : '#60a5fa'}
                  strokeWidth={isSelected ? 4 : 2}
                />

                <circle
                  r="5"
                  fill="#93c5fd"
                />

                <text
                  y="43"
                  textAnchor="middle"
                  fill="#e2e8f0"
                  fontSize="11"
                >
                  {node.account_id.length > 17
                    ? `${node.account_id.slice(0, 14)}...`
                    : node.account_id}
                </text>
              </g>
            )
          })}
        </svg>
      </div>

      <p className="graph-hint">
        Select an account to inspect its activity or select
        a relationship to inspect its transactions.
      </p>

      {selectedAccount && accountMetrics && (
        <section className="graph-details">
          <h3>Account investigation</h3>

          <strong>{selectedAccount}</strong>

          <div className="graph-metrics">
            <article>
              <span>Incoming</span>
              <strong>{accountMetrics.incomingCount}</strong>
            </article>

            <article>
              <span>Outgoing</span>
              <strong>{accountMetrics.outgoingCount}</strong>
            </article>

            <article>
              <span>Unique senders</span>
              <strong>{accountMetrics.uniqueSenders}</strong>
            </article>

            <article>
              <span>Unique recipients</span>
              <strong>{accountMetrics.uniqueRecipients}</strong>
            </article>

            <article>
              <span>Total connections</span>
              <strong>{accountMetrics.totalConnections}</strong>
            </article>
          </div>

          <button
            type="button"
            onClick={() => setSelectedAccount(null)}
          >
            Clear selection
          </button>
        </section>
      )}

      {selectedRelationship && (
        <section className="graph-details">
          <h3>Account relationship</h3>

          <p>
            <strong>{selectedRelationship.from_account}</strong>
            {' → '}
            <strong>{selectedRelationship.to_account}</strong>
          </p>

          <p>
            Transactions: {selectedRelationship.transaction_count}
          </p>

          <div className="graph-transaction-list">
            {selectedTransactions.map((transaction) => (
              <article key={transaction.transaction_id}>
                <strong>{transaction.external_reference}</strong>

                <span>
                  {transaction.currency} · {transaction.amount_minor} minor units
                </span>

                <span>
                  {transaction.transaction_type}
                </span>

                <span>
                  {new Date(
                    transaction.occurred_at,
                  ).toLocaleString()}
                </span>
              </article>
            ))}
          </div>

          <button
            type="button"
            onClick={() => setSelectedRelationship(null)}
          >
            Clear selection
          </button>
        </section>
      )}
    </div>
  )
}

export default TransactionGraphExplorer

