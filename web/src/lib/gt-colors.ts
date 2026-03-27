const GT_COLORS: Record<string, string> = {
  '0': '#ffffff', '1': '#adf4ff', '2': '#49fc00', '3': '#bfdaff', '4': '#ff271d',
  '5': '#ebb7ff', '6': '#ffca6f', '7': '#e6e6e6', '8': '#ff9445', '9': '#ffee7d',
  '!': '#d1fff9', '@': '#ffcdc9', '#': '#ff8ff3', '$': '#fffcc5', '^': '#b5ff97',
  '&': '#feeeff', 'w': '#ffffff', 'o': '#fce6ba', 'p': '#ffdff1', 'b': '#1a1a1a',
  'q': '#0c60a4', 'e': '#19b9ff', 'r': '#6fd357', 't': '#2f830d', 'a': '#515151',
  's': '#9e9e9e', 'c': '#50ffff',
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export function parseGTColors(text: string): string {
  let html = ''
  let open = false
  let i = 0
  while (i < text.length) {
    if (text[i] === '`' && i + 1 < text.length) {
      const code = text[i + 1]
      if (open) { html += '</span>'; open = false }
      if (code !== '`') {
        const color = GT_COLORS[code]
        if (color) { html += `<span style="color:${color}">`; open = true }
      }
      i += 2
    } else {
      html += escapeHtml(text[i])
      i++
    }
  }
  if (open) html += '</span>'
  return html
}
