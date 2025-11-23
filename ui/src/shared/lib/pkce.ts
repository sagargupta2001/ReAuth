// Generate a random string (Code Verifier)
export function generateCodeVerifier(): string {
  const array = new Uint32Array(56 / 2)
  window.crypto.getRandomValues(array)
  return Array.from(array, (dec) => ('0' + dec.toString(16)).slice(-2)).join('')
}

// Hash the verifier to create the Code Challenge (SHA-256)
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await window.crypto.subtle.digest('SHA-256', data)

  // Convert to Base64URL
  return btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')
}
