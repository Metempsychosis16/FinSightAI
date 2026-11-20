import { apiFetch } from '../../../shared/api/apiClient'
import type { Case } from '../../../shared/types/case'

import type {
  InvestigationDataset,
  AIInvestigationReport,
} from '../types/investigation'

import type { TransactionGraph } from '../types/transactionGraph'


export async function getCaseById(id: number): Promise<Case> {
  return apiFetch<Case>(`/cases/${id}`)
}

export async function getInvestigationDataset(
  id: number,
): Promise<InvestigationDataset> {
  return apiFetch<InvestigationDataset>(`/cases/${id}/dataset`)
}

export async function getAIReports(
  id: number,
): Promise<AIInvestigationReport[]> {
  return apiFetch<AIInvestigationReport[]>(`/cases/${id}/ai-reports`)
}

export async function getTransactionGraph(
  id: number,
): Promise<TransactionGraph> {
  return apiFetch<TransactionGraph>(`/cases/${id}/graph`)
}

