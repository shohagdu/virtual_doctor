import type { LabStatus, Report } from '../../api/types'

export const statusStyle: Record<LabStatus, { cls: string; icon: string }> = {
  normal: { cls: 'text-green-700', icon: '✔' },
  borderline: { cls: 'text-yellow-700', icon: '●' },
  high: { cls: 'text-orange-700', icon: '▲' },
  low: { cls: 'text-orange-700', icon: '▼' },
  critical: { cls: 'text-red-700 font-semibold', icon: '⚠' },
  abnormal: { cls: 'text-orange-700', icon: '+' },
}

/**
 * One Investigations line per report, listing what is not normal (FR-35), e.g.
 * "CBC: Hb 10.2 g/dL ▼, WBC 13,800 /cumm ▲". Mirrors the server's short form.
 */
export function shortLine(r: Report): string | null {
  const parts = r.results
    .filter((x) => x.status && x.status !== 'normal')
    .map((x) => [x.test_name, x.value_text, x.unit, x.status ? statusStyle[x.status].icon : ''].filter(Boolean).join(' '))
  if (parts.length === 0) return null
  return `${r.report_type?.trim() || 'Report'}: ${parts.join(', ')}`
}

/**
 * Shrinks a phone photo before upload (fewer image tokens, less mobile data; AI spec
 * §8.10): longest side at most 2000 px, JPEG. PDFs and small images pass unchanged.
 */
export async function compressImage(file: File): Promise<Blob> {
  if (!/^image\/(jpeg|png|webp)$/.test(file.type) || file.size < 1_000_000) return file
  try {
    const bitmap = await createImageBitmap(file)
    const scale = Math.min(1, 2000 / Math.max(bitmap.width, bitmap.height))
    const canvas = document.createElement('canvas')
    canvas.width = Math.round(bitmap.width * scale)
    canvas.height = Math.round(bitmap.height * scale)
    canvas.getContext('2d')?.drawImage(bitmap, 0, 0, canvas.width, canvas.height)
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, 'image/jpeg', 0.85))
    return blob && blob.size < file.size ? blob : file
  } catch {
    return file
  }
}
