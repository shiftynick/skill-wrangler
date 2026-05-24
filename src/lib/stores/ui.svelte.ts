export const uiState = $state({
  error: null as string | null,
  compactList: false,
});

export function setError(message: string | null): void {
  uiState.error = message;
}
