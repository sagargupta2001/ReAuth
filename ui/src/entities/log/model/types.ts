export interface LogEntry {
  timestamp: string
  level: string
  target: string
  message: string
  fields: Record<string, string>
}
