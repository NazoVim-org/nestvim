type CommandHandler = () => void | Promise<void>;

export class CommandRegistry {
  private commands = new Map<string, CommandHandler>();

  register(name: string, handler: CommandHandler): void {
    if (this.commands.has(name)) {
      console.warn(`[CommandRegistry] Command "${name}" is already registered. Overwriting.`);
    }
    this.commands.set(name, handler);
  }

  async execute(name: string): Promise<boolean> {
    const handler = this.commands.get(name);
    if (!handler) return false;
    await handler();
    return true;
  }

  has(name: string): boolean {
    return this.commands.has(name);
  }

  list(): string[] {
    return [...this.commands.keys()];
  }
}