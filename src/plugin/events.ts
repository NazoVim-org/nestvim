import type {
  EditorEventMap,
  EditorEventName,
  EditorEventPayload,
} from "./types";

type Handler<T extends EditorEventName> = (
  payload: EditorEventPayload<T>
) => void;

export class EventEmitter {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private listeners = new Map<EditorEventName, Set<Handler<any>>>();

  on<T extends EditorEventName>(event: T, handler: Handler<T>): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(handler);
  }

  off<T extends EditorEventName>(event: T, handler: Handler<T>): void {
    this.listeners.get(event)?.delete(handler);
  }

  async emit<T extends EditorEventName>(
    event: T,
    payload: EditorEventPayload<T>
  ): Promise<void> {
    const handlers = this.listeners.get(event);
    if (!handlers) return;
    for (const handler of handlers) {
      try {
        await handler(payload);
      } catch (err) {
        process.stderr.write(`[EventEmitter] Error in handler for "${event}": ${err}\n`);
      }
    }
  }

  clear(): void {
    this.listeners.clear();
  }
}