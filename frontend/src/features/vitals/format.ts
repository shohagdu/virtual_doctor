import type { Vitals } from '../../api/types'

/** Vitals as short "label value unit" parts, e.g. ["BP 120/80 mmHg", "Pulse 72/min"]. */
export function vitalsParts(v: Vitals | null | undefined): string[] {
  if (!v) return []
  const parts: (string | null)[] = [
    v.bp_sys !== null && v.bp_dia !== null ? `BP ${v.bp_sys}/${v.bp_dia} mmHg` : null,
    v.pulse !== null ? `Pulse ${v.pulse}/min` : null,
    v.temp !== null ? `Temp ${v.temp} °F` : null,
    v.weight !== null ? `Wt ${v.weight} kg` : null,
    v.height !== null ? `Ht ${v.height} cm` : null,
    v.bmi !== null ? `BMI ${v.bmi}` : null,
    v.spo2 !== null ? `SpO₂ ${v.spo2}%` : null,
    v.rbs !== null ? `RBS ${v.rbs} mmol/L` : null,
  ]
  return parts.filter((p): p is string => p !== null)
}
