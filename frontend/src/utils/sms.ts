/** SMS parts needed: 160/153 chars for plain text, 70/67 for Unicode (Bangla). Mirrors the server. */
export function smsSegments(text: string) {
  const n = [...text].length
  // eslint-disable-next-line no-control-regex
  const ascii = /^[\x00-\x7F]*$/.test(text)
  const [single, multi] = ascii ? [160, 153] : [70, 67]
  return n <= single ? 1 : Math.ceil(n / multi)
}
