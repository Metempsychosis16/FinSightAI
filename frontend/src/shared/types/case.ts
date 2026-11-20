export type CaseStatus =
  | 'OPEN'
  | 'UNDER_REVIEW'
  | 'ESCALATED'
  | 'CLOSED'
  | 'ARCHIVED'

export interface Case {
  id: number
  title: string
  risk_score: number
  status: CaseStatus
  created_at: string
  updated_at: string
}
