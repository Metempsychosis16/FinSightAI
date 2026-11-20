import type { Case } from '../../../shared/types/case'

export interface Investigator {
  id: number
  user_id: number
  name: string
  department: string
  years_experience: number
}

export interface DatasetTransaction {
  id: number
  external_reference: string
  amount_minor: number
  currency: string
  sender_account: string
  recipient_account: string
  transaction_type: string
  occurred_at: string
  linked_at: string
}

export interface Evidence {
  id: number
  case_id: number
  title: string
  description: string
  evidence_type: string
  source_reference: string
  sha256_hash: string | null
  collected_by: number
  created_at: string
}

export interface InvestigationDataset {
  case: Case
  investigator: Investigator | null
  transaction_count: number
  transactions: DatasetTransaction[]
  evidence_count: number
  evidence: Evidence[]
}

export interface AIInvestigationReport {
  id: number
  case_id: number
  generated_by: number
  provider: string
  model: string
  prompt_version: string
  schema_version: string
  structured_report:  unknown
  markdown_report: string
  created_at: string
}
