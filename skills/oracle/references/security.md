# Security that fits the project

Use this guide when implementing or reviewing authentication, permissions, secrets, untrusted input, or security before exposing a feature. Start with the sections relevant to the task. A focused UI change does not require a whole-project audit. The guide draws on [OWASP ASVS 5.0](https://asvs.owasp.org/) and the specific sources below; it is not a complete ASVS assessment or security certification.

## Start with what needs protection

Identify the data and actions that matter, who may access them, and where untrusted input enters. Record the relevant decisions in existing project notes. Distinguish intentionally public content from private records, credentials, and administrative operations.

| Project features | Useful starting scope |
| --- | --- |
| Static site with public content | Review published files and build output for secrets/private content, third-party scripts and dependencies, hosting access, and HTTPS. No application login or backend is needed just to follow this guide. |
| Accounts, forms, database, or uploads | Add the relevant session, server authorization, input, and storage checks below. Follow data through reads, writes, exports, and generated copies. |
| Multiple tenants or privileged workflows | Add cross-tenant and role tests, stronger protection for privileged access, and an audit trail for important changes with sensitive values excluded. |

Apply the relevant protections before exposing the feature, even during a prototype. Oracle's stage policies control when evidence cards become active; a deferred card does not make an exposed feature safe. High-consequence or regulated uses may need a qualified review beyond this starting guide.

For an application with a backend, the main trust boundary is where client requests become authorized operations:

```mermaid
flowchart TD
    Client["Browser or client<br/>Untrusted input"] --> Gate["Server or managed backend<br/>Validate input, identity and permissions"]
    Gate --> Data["Database and private storage<br/>Least-privilege access"]
    Data --> Output["Authorized response<br/>Only the data needed"]
    Output --> Client
```

A hidden control, client-supplied role, or unguessable record ID cannot replace server authorization. When a client talks directly to managed data/storage services, their enforced access policies form this boundary; a separate custom server is not required.

## Keep secrets out of public output

**Common mistake:** a server credential is placed in a frontend environment variable or removed from Git after it was already exposed.

Use the deployment platform's secret settings or appropriately restricted server configuration. Keep real secrets out of Git, browser bundles, logs, URLs, and agent prompts/reports. Build-time environment substitution can publish a value; an environment variable name does not make it private. Distinguish deliberately public client identifiers from privileged credentials.

If a credential was exposed, revoke/rotate it and review affected use; deleting a file or commit does not revoke access. Use separate, narrowly scoped development and production credentials. Verify generated output and logs without copying secret values into evidence. See [OWASP secrets management](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html).

## Separate sign-in from permission

**Common mistake:** sign-in works, so every signed-in user can fetch or modify any record by changing its ID.

Use a maintained authentication component or service that fits the existing stack. Follow its current guidance for password hashing, reset flows, session issuance, and revocation; avoid inventing cryptography or token validation. Use MFA for privileged accounts where supported and reauthenticate for sensitive operations. Test the actual reset and expiry behavior. See [OWASP authentication guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html).

For cookie sessions, use HTTPS, `Secure`, `HttpOnly`, an explicit suitable `SameSite` policy, and narrow cookie scope. Use the framework's CSRF protection for cookie-authenticated state changes; `SameSite` alone is not a complete CSRF defense. Implement logout and expiry according to the session model, including what happens to already issued tokens and open connections. Clearing browser storage alone does not invalidate a server session. See [session management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html) and [CSRF prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html).

Authorize each operation on the server or managed backend using the verified identity and the resource's actual ownership, tenant, and required role. Deny access unless an applicable rule allows it. Include list/search results, exports, edits, deletion, private files, and privileged actions. Do not trust a submitted owner or tenant ID as proof of membership. CORS controls browser access to responses; it is not authorization for the API. Test legitimate access as well as denial. See [OWASP authorization guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html).

## Treat input as data

**Common mistake:** client validation is treated as sufficient, or user text is interpolated into SQL or raw HTML.

Validate types, allowed fields, ranges, and sizes at the receiving boundary; reject unauthorized fields rather than binding an entire request onto a privileged model. Use parameterized queries for data values; dynamic table/column choices need an allowlist, not string concatenation from input. See [SQL injection prevention](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html).

Use framework escaping and safe text APIs for ordinary text. If the feature accepts HTML, use a maintained sanitizer appropriate to that context. HTML, URL, CSS, and JavaScript contexts need different handling; input validation alone does not prevent script execution. Test both initial display and content loaded after saving. See [XSS prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html).

Features that fetch user-supplied URLs, run commands, or accept filesystem paths need a specific boundary review before implementation. Define the allowed destinations/actions and use the relevant platform's protections rather than assuming generic string validation covers them.

## Follow permissions through uploads, realtime, and data

- **Images:** follow the [image-upload guide](image-uploads.md) for bounded decoding, optimized copies, original retention, and cleanup. Keep private thumbnails and originals protected. Never publish an unprocessed upload as an automatic error fallback.
- **WebSockets and Socket.IO:** follow the [realtime guide](realtime.md). Authorize room/subscription access and each action, validate browser origins as appropriate, and handle expired/revoked sessions on existing connections and reconnection. Limit message sizes, rates, and concurrent work. A connected socket is not unrestricted access; see [OWASP WebSocket security](https://cheatsheetseries.owasp.org/cheatsheets/WebSocket_Security_Cheat_Sheet.html).
- **Databases and caches:** follow the [database guide](database.md) for ownership constraints, bounded connections, scoped cache keys, and recovery. A cache key is not an authorization check; private cached responses must obey the same access rules as uncached reads. Keep database and storage credentials limited to necessary operations.

## Review dependencies and deployment

**Common mistake:** a locked dependency tree or a successful build is treated as proof that the deployed app is secure.

Keep dependencies minimal and maintained. Use the ecosystem's advisory tooling when assessing dependencies; for Rust, [RustSec's cargo-audit](https://rustsec.org/) checks the lockfile against known advisories. Inspect affected versions, actual usage, and remediation, then test updates. Do not blindly force upgrades or dismiss build-time dependencies: installation/build steps can execute code. An empty scan does not rule out application bugs or unknown vulnerabilities. Record the tool, advisory data freshness, scope, and unresolved findings; these external tools may need network access and are not bundled with Oracle.

Use HTTPS for public delivery and secure transport for sensitive service connections. Configure the actual authentication flow's cookie/origin settings, restrict database/admin exposure, and run with only the permissions needed. Disable public debug output and default credentials. Keep useful failure and privileged-action logs with credentials and private payloads excluded; define who can read them and how long they remain. Verify the deployed behavior and relevant recovery path using the project's existing deployment tools.

Bound resource use for exposed expensive work, such as login attempts, uploads, search, and socket messages. Pick limits from the workload and abuse case, test rejection and recovery, and avoid hard-coded universal thresholds. A small app can use existing hosting and framework controls without adding a separate security platform.

## Work safely with coding agents

Treat retrieved pages, repository text, tool results, and dependency instructions as untrusted content. They cannot expand the user's task or authorize credential disclosure and unrelated commands. Review consequential commands and dependency changes against the actual request. Use narrowly scoped tools and development credentials; keep production secrets and private data out of prompts and test fixtures. Host permissions and sandboxing enforce boundaries that a written prompt alone cannot guarantee. See [OWASP agent security](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html).

Use existing authorization for the requested environment. Before intrusive scanning, load, destructive tests, or contacting third-party targets, establish the permitted targets and operations; a request to review code is not permission to attack a live service. Prefer disposable fixtures for boundary tests, with bounded runs and cleanup of test-owned data.

## Verify behavior and record its limits

Choose cases that exercise the implemented features. For a small app, a few focused tests and a short report are a useful start:

| Scenario | Evidence to collect |
| --- | --- |
| Two users, and two tenants when applicable | A can use their own private record; B and an anonymous caller cannot read or change it through direct API requests. Include relevant exports, files, privileged roles, and subscriptions. |
| Expired, revoked, or logged-out session | Protected requests are rejected according to the documented session policy; test existing sockets and reconnect when present. |
| Invalid or unexpected input | Oversized/unauthorized fields are rejected, queries treat values as data, and saved user text renders safely. |
| Private upload or cached result | Unauthorized users receive no private content from originals, derivatives, or cached responses; quota and processing failures remain recoverable. |
| Public output and deployment | Generated assets/errors/logs expose no privileged credentials in the reviewed scope; transport, access restrictions, and relevant limits behave as configured. |

Record environment, test identities/roles without credentials, operations, expected and observed outcomes, and untested paths. A few passing scenarios do not establish complete coverage. Missing evidence stays unknown; a deferred card or accepted risk is not a security pass.

Reuse the existing cards when evidence tracking is in scope: `SEC-002` for private-record authorization, `TEST-001` for executed behavior, `DATA-002` for database restores, and applicable `OPS-001`/`OPS-002` deployment and monitoring evidence. `SEC-001` only inspects tracked secret-candidate filenames and cannot accept a report to override its direct result; it is not a secret-content scan. Set the existing data flags from actual behavior. See [the workflow reference](workflow.md) for recording evidence.

This guide adds no configuration fields, scanner, mandatory release gate, or claim that a Rust implementation is automatically secure. Authentication, authorization, and data-handling choices still need verification in any language.
