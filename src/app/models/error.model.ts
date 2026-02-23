export interface AppError {
  code: string;
  message: string;
}

export function extractError(e: unknown): AppError {
  if (e !== null && typeof e === 'object' && 'code' in e && 'message' in e) {
    return e as AppError;
  }
  if (typeof e === 'string') {
    return { code: 'UNKNOWN', message: e };
  }
  if (e instanceof Error) {
    return { code: 'UNKNOWN', message: e.message };
  }
  return { code: 'UNKNOWN', message: 'An unexpected error occurred' };
}
