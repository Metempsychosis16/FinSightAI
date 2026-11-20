import { apiFetch } from '../../../shared/api/apiClient'
import type { Case } from '../../../shared/types/case'

export async function getCases(): Promise<Case[]> {
  return apiFetch<Case[]>('/cases')
}

export async function getHighRiskCases(): Promise<Case[]> {
  return apiFetch<Case[]>('/cases/high-risk')
}
