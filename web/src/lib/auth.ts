const TOKEN_KEY = 'mori_token'

export const authStore = {
  getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY)
  },
  setToken(token: string) {
    localStorage.setItem(TOKEN_KEY, token)
  },
  clearToken() {
    localStorage.removeItem(TOKEN_KEY)
  },
  isLoggedIn(): boolean {
    return !!localStorage.getItem(TOKEN_KEY)
  },
}
