<!-- README-SOURCE-SHA256: ae8dd6510a35351cf2f36a3f9ee929801f298509bd78a84ceed1e14535da3640 -->

<p align="center">
  <img src="../../assets/memorywhale-logo-sm.png" alt="Logotipo de MemoryWhale" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Memoria de depuración local y persistente para desarrolladores y agentes de código.</strong></p>

<p align="center" dir="ltr"><a href="../../README.md">English README</a> · <a href="../../docs/i18n/README.ar.md" lang="ar">العربية</a> · <a href="../../docs/i18n/README.de.md" lang="de">Deutsch</a> · <a href="../../docs/i18n/README.fr.md">README français</a> · <a href="../../docs/i18n/README.zh-CN.md">简体中文 README</a> · <a href="../../docs/i18n/README.zh-TW.md">繁體中文 README</a> · <a href="../../docs/i18n/README.ko.md">한국어 README</a> · <a href="../../docs/i18n/README.ja.md">日本語 README</a> · <a href="../../docs/i18n/README.es.md" lang="es">Español</a> · <a href="../../docs/i18n/README.pt-BR.md" lang="pt-BR">Português (Brasil)</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="versión"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="licencia MIT"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local primero, no se sube nada"/>
</p>

<p align="center">
  <img src="../../assets/recall-demo.gif" alt="Vuelve a aparecer un error de compilación; mw context indica que ya ocurrió dos veces y mw search devuelve la solución guardada." width="820" />
</p>

**Te topas con un error. MemoryWhale recuerda que ya lo viste antes y te devuelve la solución.**

MemoryWhale registra lo que realmente ocurrió mientras depurabas: comandos,
salida, fallos y las soluciones que funcionaron. Guarda esa evidencia en SQLite
local para que tú y tus agentes de código puedan encontrarla cuando el terminal,
la conexión SSH o la sesión del agente ya no estén. Funciona con 19 agentes de
código a través de MCP, sin cuenta y sin subir nada.

> En [LongMemEval](../../benchmarks/longmemeval/README.md), un benchmark público de memoria a largo plazo, la búsqueda sitúa
> una sesión que contiene la respuesta entre sus 5 primeros resultados en el **97%**
> de 470 preguntas (solo recuperación, sin modelo).
>
> En una evaluación controlada con bugs específicos de un proyecto, un agente resolvió
> el **25%** sin memoria y el **96%** con ella. Son tareas sintéticas que
> demuestran el mecanismo, no un estudio de campo; consulta
> [cómo se midió](../../benchmarks/README.md).

**MemoryWhale 0.16.0: Proactive Recall · 2 de octubre de 2026.**
La CLI, la interfaz web y la aplicación de escritorio comparten la versión de
producto 0.16.0; el núcleo reutilizable en Rust está en la versión 0.8.0. Consulta las [notas de la versión](https://github.com/wuisabel-gif/MemWhale/blob/v0.16.0/docs/releases/0.16.0.md)
para ver la guía de actualización. Al abrir un almacén, se migra del esquema 10 al 13.

**¿Quieres contribuir?** Empieza por el [issue «Start here»](https://github.com/wuisabel-gif/MemWhale/issues/317).
Muchas tareas no requieren Rust, como revisar traducciones o corregir la documentación.

## Por qué MemoryWhale

- **Recuerda lo que realmente ocurrió.** Conserva el comando, el entorno, la
  salida, el fallo y la lección, no solo una línea del historial del shell.
- **Usa una sola memoria con todos tus agentes de código.** Cualquier cliente
  MCP stdio compatible puede leer y escribir en la misma memoria local a través de `mw-mcp`.
- **Mantén local tu historial de desarrollo.** MemoryWhale funciona sin cuenta,
  sin servicio alojado y sin pagar la memoria por token.

MemoryWhale registra tu experiencia de desarrollo, no todo lo que haces. Es una
capa de memoria para depurar, no un agente de código autónomo, ni un sistema de
memoria personal de uso general, ni un sustituto de la documentación del proyecto.

## Novedades de Agent-Native Memory

- **Conecta e inspecciona agentes.** Instala el acceso MCP para Claude Code o Rho,
  los hooks de captura y las pautas de uso de la memoria con `mw integrate`;
  `mw doctor` comprueba MCP, hooks y skills de forma independiente.
- **Mantén explícita la procedencia.** El esquema 10 guarda el agente de cada
  comando como `claude`, `rho` o `NULL`. La etiqueta `terminal`, usada al mostrar
  y filtrar, indica una procedencia de terminal o manual, o heredada (legacy); no
  demuestra que lo haya ejecutado una persona. La identidad del agente es
  independiente del tipo de origen, como `command`, `session` o `note`.
- **Comparte un repositorio, distingue los worktrees.** Los identificadores
  canónicos de repositorio agrupan los worktrees vinculados y conservan la raíz
  de cada worktree y las etiquetas de proyecto existentes. La detección lee los
  metadatos locales de Git, no un servicio remoto.
- **Usa interfaces locales.** `mw-serve` ofrece MCP por HTTP en `POST /mcp`;
  `mw-serve --api` activa, solo si lo pides, la API JSON de solo lectura. Ambas
  usan el listener del panel; el acceso fuera de loopback requiere un token.
- **Obtén el contexto de GitHub de forma explícita.** `mw github context <pr>` lee
  los metadatos, checks y revisiones de un PR mediante tu sesión de `gh` ya
  iniciada. Muestra un contexto acotado y con datos sensibles ocultos, sin hacer
  checkout del código ni guardarlo automáticamente en la memoria. No hay
  sincronización con GitHub en segundo plano.

## Instalación

Hay binarios precompilados para Linux x86_64/aarch64 y macOS:

```bash
(
  set -eu
  installer="$(mktemp)"
  trap 'rm -f "$installer"' EXIT
  curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/7c3864c743cec9a8fa813dcc0b2459cc2859c849/install.sh -o "$installer"
  printf '%s  %s\n' '3e0cad72b29c1894d5ff5f7c30b099537f96501801c14b6320c12e169a3ac8d6' "$installer" | shasum -a 256 -c -
  sh "$installer"
)
```

También puedes instalarlo con Cargo o Homebrew:

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Después de instalar o actualizar, comprueba la versión y la configuración local:

```bash
mw --version
mw doctor
```

En Windows, puedes usar MemoryWhale dentro de
[WSL](https://learn.microsoft.com/windows/wsl/). Consulta la
[guía de inicio](../../docs/guides/getting-started.md) para la instalación con paquetes,
la configuración del PATH y las notas de cada plataforma.

## Ejemplo en sesenta segundos

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![Demostración de los estados de ánimo de mw pet](../../assets/pet-demo.gif)

Para trabajos más largos, `mw --live` graba una sesión de shell resistente a
caídas. `mw tui` abre un navegador interactivo en el terminal, y `mw-serve`
inicia el panel web local.

## Cómo funciona

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

La captura y la recuperación son independientes. MCP da a un agente acceso a la
memoria existente; no registra automáticamente la actividad normal del terminal.
Consulta la [arquitectura](../../docs/architecture.md) y el
[concepto de captura](../../docs/concepts/capture.md) para ver el modelo completo.

## Funciona con tu agente de código

`mw-mcp` es el punto de integración común: un servidor MCP stdio local que expone
seis herramientas de memoria, también disponibles por HTTP a través de `mw-serve`.
Hay guías para Claude Code, Rho, Claude Desktop, Cursor, VS
Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue, Gemini CLI,
Goose, OpenClaw, CrowClaw, Hermes Agent y otros clientes compatibles.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

No todos los clientes ofrecen las mismas capacidades. MCP permite acceder a la
memoria; la captura automática de ejecuciones requiere un hook específico del
cliente. La [matriz de integración](../../integrations/README.md) distingue acceso,
captura y pautas de uso de la memoria, y enlaza cada guía de configuración verificada.

El payload actual de los hooks de Rho no incluye el texto del comando ni stdout:
los fallos pueden registrarse como metadatos con un comando centinela, y las
llamadas correctas sin texto de comando se omiten. La [demo de traspaso entre agentes](../../docs/guides/cross-agent-handoff.md)
usa fixtures y un cliente Rho simulado contra un MCP real, no agentes en vivo
ni una solución de Cargo verificada.

La skill incluida orienta el uso de la memoria; no implementa la recuperación
automática al iniciar una tarea, la búsqueda de fallos ni el guardado previo a
la compactación. Esas decisiones de ciclo de vida corresponden al cliente. Las
lecciones creadas a través de MCP quedan pendientes de revisión por defecto.

## ¿Para quién es MemoryWhale?

MemoryWhale es para desarrolladores cuyo contexto de depuración está repartido
entre el historial de desplazamiento del terminal, el historial del shell,
distintas máquinas y sesiones temporales de agentes. Es especialmente útil si:

- depuras compilaciones, dependencias, Git, entornos o despliegues;
- usas agentes de código en varias sesiones o cambias de una herramienta a otra;
- trabajas por SSH o en varias máquinas de desarrollo;
- quieres que los fallos recurrentes y sus soluciones se puedan seguir buscando;
- prefieres el almacenamiento local a un servicio de memoria alojado.

Consulta los [casos de uso](../../docs/concepts/use-cases.md), donde cada uno se
describe como un escenario completo con comandos reales.

## Documentación

- [Mapa de la documentación](../../docs/README.md)
- [Primeros pasos](../../docs/guides/getting-started.md)
- [Referencia de `mw pet`](../../docs/reference/pet.md)
- [Captura del terminal](../../docs/guides/terminal-capture.md)
- [Memoria para agentes](../../docs/guides/agent-memory.md)
- [Referencia de la CLI](../../docs/reference/cli.md)
- [API JSON local](../../docs/reference/api.md)
- [Referencia de MCP](../../docs/reference/mcp.md)
- [Seguridad y modelo de amenazas local](../../docs/SECURITY.md)
- [Ecosistema](../../ECOSYSTEM.md): Delphin, ContextGC y MemoryWhale juntos
- [Guías de integración y matriz de capacidades](../../integrations/README.md)

## Contribuir

MemoryWhale acepta cambios que mejoren la captura, la conservación, la
recuperación o el intercambio de experiencia de desarrollo. Lee
[CONTRIBUTING.md](../../CONTRIBUTING.md) para conocer la regla de alcance, los comandos
de desarrollo y la lista de comprobación de pull requests. Si contribuyes por
primera vez, elige una tarea del [issue «Start here»](https://github.com/wuisabel-gif/MemWhale/issues/317).

Cada pull request recibe además una revisión automática de
[Second-Opinion](https://github.com/wuisabel-gif/second-opinion), una GitHub Action de un proyecto
hermano del mismo mantenedor. Sus comentarios son sugerencias; una persona decide
qué aplicar. La configuración y los límites están en la [guía de Second-Opinion](../../integrations/second-opinion/README.md).

Publicado bajo la [licencia MIT](../../LICENSE).
