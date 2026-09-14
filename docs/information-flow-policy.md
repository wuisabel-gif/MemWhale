# Information-flow security constitution

Status: **design policy; not implemented enforcement**.

This document defines the rules that any future confidentiality controls in
MemoryWhale must satisfy. It is inspired by Bell–LaPadula, adapted to a local
debugging-memory system that can send selected records to external clients and
model providers.

Adding a label to a database row does not by itself enforce this policy. Until
all prerequisites below are implemented and tested, documentation and user
interfaces must not claim Bell–LaPadula enforcement.

## Security objective

Prevent information from flowing from a more confidential source to a less
trusted reader or destination without an explicit, authorized declassification.
The initial ordered lattice is:

```text
public < internal < sensitive < restricted
```

A higher level dominates every lower level. Labels are canonical lowercase
values; aliases and unknown labels are invalid. Compartments such as separate
organizations or projects are intentionally deferred: ordering alone cannot
express all isolation requirements.

## Normative rules

The words **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are normative.

1. **No read up.** A subject MUST NOT read a record whose classification is
   higher than the subject's effective clearance.
2. **No write down.** Information MUST NOT be written to a destination below
   its effective classification.
3. **Complete mediation.** Capture, storage, search, context generation, MCP,
   APIs, exports, and client integrations MUST apply the same policy at every
   access; filtering only one interface is insufficient.
4. **Explicit identities.** Authorization MUST use authenticated or otherwise
   trustworthy subject and destination identities. A process name, agent label,
   hook presence, filesystem path, or model-supplied field is not identity.
5. **Default deny.** Missing identity, clearance, destination trust,
   classification, or malformed policy MUST NOT broaden access.
6. **Provider mediation.** Permission to read locally does not imply permission
   to send the result to a hosted model. Provider/export clearance MUST be
   checked independently.
7. **Non-disclosure on denial.** A denial response MUST NOT reveal protected
   content through excerpts, record IDs, counts, ranking, error differences, or
   avoidable timing differences.
8. **Monotonic handling.** Derived summaries, indexes, embeddings, logs, and
   cached context MUST inherit at least the highest classification of their
   inputs unless an explicit declassification operation applies.
9. **Explicit declassification.** Declassification MUST require a deliberate
   authorized action, identify the affected records and destination, and create
   a content-free audit event. A model cannot authorize declassification.
10. **Separate integrity.** Confidentiality classification MUST NOT represent
    provenance, correctness, confirmation, or verification state.

## Entities and decisions

- **Object:** a command run, output, note, lesson, transcript fragment, derived
  index entry, summary, count, or exported context.
- **Subject:** a local user operation, authenticated API caller, MCP client,
  capture adapter, or another explicitly identified principal.
- **Destination:** the store, response channel, client, model provider, export
  file, or machine receiving information.
- **Effective classification:** the object's label combined with labels of all
  source material used to derive it.
- **Effective clearance:** the maximum classification a subject or destination
  may receive for this operation. A caller MAY voluntarily lower its effective
  clearance, but cannot raise it through request input.

Subject clearance and destination clearance are separate. For example, a local
operator may be cleared for `restricted` data while an external model endpoint
is approved only for `public` data.

## Information-flow table

| Boundary | Required decision | Required behavior when unavailable |
| --- | --- | --- |
| Capture → memory | Classify input and authorize destination write | Do not persist; return or locally audit a content-free denial |
| Memory → retrieval | Compare every object with subject clearance | Exclude without leaking existence; fail closed if policy cannot be evaluated |
| Retrieval → response | Classify aggregates and derived context | Preserve the highest input classification |
| Response → MCP/API/client | Authenticate subject and response destination | Deny; never trust caller-supplied agent names alone |
| Client → model provider | Compare result classification with provider/export clearance | Do not transmit protected content |
| Import/export → another store | Check source labels, destination policy, and declassification authority | Abort without partial downgrade |

## Examples

| Operation | Decision | Reason |
| --- | --- | --- |
| `sensitive` local user reads `internal` record | Allow | Clearance dominates object |
| `internal` MCP client requests `sensitive` record | Deny | No read up |
| `sensitive` command output is saved as `public` | Deny | No write down |
| `restricted` operator searches locally but sends results to a `public` provider | Deny export | Local and provider decisions are separate |
| Summary combines `public` and `sensitive` records | Label `sensitive` | Derived data inherits the highest input label |
| Search has one hidden match above caller clearance | Return the same observable shape as no authorized match | Do not disclose existence through counts or errors |
| User explicitly requests a lower clearance for a command | Allow the lower effective clearance | Voluntary restriction cannot increase access |
| Model output asks to relabel a record as public | Deny | A model cannot authorize declassification |

## Compatibility defaults to decide before implementation

No runtime default is approved by this document. The implementation design MUST
choose and security-review all of the following together:

- classification assigned to pre-migration records;
- classification assigned to new records when no label is supplied;
- clearance for existing CLI calls and clients without identity metadata;
- treatment of mixed-version databases and older binaries;
- whether migration is reversible and how rollback avoids label loss;
- behavior when policy configuration is missing, unreadable, or downgraded.

Choosing `public` for compatibility risks disclosure. Choosing `restricted`
reduces availability. Neither choice may be introduced implicitly.

## Minimum implementation prerequisites

Before enforcement work begins, a proposal MUST define:

1. A trusted local policy-administration boundary and file permissions.
2. Subject authentication for each public interface, not just an `agent` filter.
3. Destination/provider identity and clearance propagation.
4. Durable labels for source and derived objects, including indexes and caches.
5. Atomic migration and rollback behavior for existing databases.
6. Uniform authorization APIs used by Capture, Memory, Retrieval, and
   Interfaces rather than client-specific checks.
7. Content-free, bounded audit records for policy decisions.
8. Tests for no-read-up, no-write-down, mixed results, missing identity,
   malformed policy, declassification, provider export, and non-disclosure.
9. A versioned compatibility contract for older clients and stores.
10. Independent security review before documentation says enforcement exists.

## Integrity and verification are different

Bell–LaPadula protects confidentiality, not truth. A `restricted` model summary
may still be wrong. A command exiting zero does not verify its explanation.
Future integrity rules should separately represent states such as `unverified`,
`observed`, `reviewed`, and `verified`, using a Biba-inspired policy if useful.
Promotion must require appropriate evidence and authorization.

Provenance, integrity/verification, and confidentiality therefore remain three
separate dimensions. No one field may silently stand in for another.

## Threat model and limits

This policy is intended to reduce accidental cross-project disclosure,
unauthorized retrieval, unsafe provider export, identity spoofing at public
interfaces, and leakage through result metadata.

It does not replace operating-system permissions, encryption, secret rotation,
provider-side controls, sandboxing, or host security. A fully compromised
process running with the user's filesystem authority can bypass application
logic. Prompt-injection text in stored records remains untrusted even when the
reader is cleared to see it.

## Change control

Changes to the lattice, defaults, identity model, declassification rules, or
non-disclosure behavior require architectural and security review. Runtime or
schema work must link to that approved decision and include migration and
negative-test evidence. Convenience and backward compatibility do not override
the default-deny and no-downgrade rules.
