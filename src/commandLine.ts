export function splitCommandLine(commandLine: string): string[] {
  return commandLine
    .match(/(?:[^\s"']+|"[^"]*"|'[^']*')+/g)
    ?.map((part) => part.replace(/^['"]|['"]$/g, "")) ?? [];
}
