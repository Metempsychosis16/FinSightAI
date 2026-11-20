
export interface AccountNode {
  account_id: string
}

export interface TransactionEdge {
  transaction_id: number
  external_reference: string
  from_account: string
  to_account: string
  amount_minor: number
  currency: string
  transaction_type: string
  occurred_at: string
}

export interface AccountRelationship {
  from_account: string
  to_account: string
  transaction_count: number
  transaction_ids: number[]
}

export interface TransactionGraph {
  case_id: number
  nodes: AccountNode[]
  edges: TransactionEdge[]
  relationships: AccountRelationship[]
  adjacency_list: Record<string, string[]>
}

