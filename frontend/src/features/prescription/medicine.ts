import type { MedicineLine } from '../../api/types'

/**
 * Medicines whose brand or generic name matches a recorded allergy (PM-09).
 * A plain text match: it catches "Penicillin" in "Phenoxymethylpenicillin" but not
 * drug-class relations, which need the medicine database (Phase 2).
 */
export function allergyMatches(medicines: MedicineLine[], allergies: string[]): { brand: string; allergy: string }[] {
  const out: { brand: string; allergy: string }[] = []
  for (const m of medicines) {
    const names = [m.brand, m.generic ?? ''].map((s) => s.toLowerCase()).filter((s) => s.length >= 3)
    for (const a of allergies) {
      const al = a.toLowerCase().trim()
      if (al.length >= 3 && names.some((n) => n.includes(al) || al.includes(n))) {
        out.push({ brand: m.brand, allergy: a })
      }
    }
  }
  return out
}

export const emptyLine: MedicineLine = {
  medicine_id: null,
  brand: '',
  generic: null,
  strength: null,
  form: null,
  dose_pattern: null,
  timing: null,
  duration: null,
  notes: null,
}
