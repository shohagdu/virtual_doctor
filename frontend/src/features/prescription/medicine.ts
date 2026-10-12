import type { MedicineLine } from '../../api/types'

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

/** "Tab Napa 500 mg" */
export function medicineName(m: MedicineLine) {
  return [m.form, m.brand, m.strength].filter(Boolean).join(' ')
}

/** "1+1+1 — খাবারের পরে — ৫ দিন" */
export function medicineDirections(m: MedicineLine) {
  return [m.dose_pattern, m.timing, m.duration].filter(Boolean).join(' — ')
}

const sameMedicine = (a: MedicineLine, b: MedicineLine) =>
  a.brand.trim().toLowerCase() === b.brand.trim().toLowerCase() &&
  (a.strength ?? '').trim().toLowerCase() === (b.strength ?? '').trim().toLowerCase()

/**
 * Adds copied medicines (Repeat ℞ / Copy from history, PM-10) after the current ones,
 * dropping blank rows and medicines that are already on the prescription.
 */
export function addMedicines(current: MedicineLine[], copied: MedicineLine[]): MedicineLine[] {
  const kept = current.filter((m) => m.brand.trim())
  const fresh = copied.filter((m) => m.brand.trim() && !kept.some((k) => sameMedicine(k, m)))
  return [...kept, ...fresh]
}
