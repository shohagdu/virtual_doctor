import type { PatientBrief } from '../api/types'

/** "Mr Rahim Uddin": title and English name. */
export function fullName(p: Pick<PatientBrief, 'title' | 'name_en'>) {
  return [p.title, p.name_en].filter(Boolean).join(' ')
}
