# Engineering Standards for AI Agents & Automated Contributors (AGENTS.md)

This document establishes the binding engineering guidelines for any **Autonomous Agent**, **LLM-assisted developer**, or **Automated CI/CD Workflow** submitting issues, pull requests, or code reviews to the **Aurexalis** repository.

---

## 🧭 Core Principles

1. **Human Ownership & Accountability**:
   Automated agents and AI models do not hold legal or technical accountability. The human author whose credentials push the code or open the PR assumes **full responsibility** for the correctness, performance, security, and maintenance of the changes.

2. **Transparent Disclosure**:
   In accordance with open source best practices, contributors must explicitly disclose the involvement of AI agents or LLMs in the PR description:
   ```markdown
   ### 🤖 AI Assistance Disclosure
   - Tools used: [e.g., Claude Code, Cursor, Copilot, Custom Agent]
   - Human verification: Fully reviewed, locally compiled, and all unit tests passed.
   ```

3. **Zero AI Slop & Anti-Hallucination Policy**:
   PRs containing any of the following will be **immediately closed without review, marked as spam, and may result in a contributor block**:
   * Hallucinated Rust crates, fake Gecko/Firefox internal APIs, or non-existent `.uc.js` hooks.
   * "Plumbing without water": empty boilerplate files, dummy implementations, or unneeded scaffolding.
   * Unrequested refactoring of working browser modules without functional or performance justification.
   * Broken builds, compiler warnings (`cargo clippy`), or failing tests.
   * Circular, repetitive LLM apologies in issue or review threads.

---

## 🧪 Verification & Evidence Requirement

Every PR submitted to Aurexalis with AI assistance **must include verifiable evidence of local execution**:

* **Rust Workspace**:
  - `cargo check --workspace --all-features` must pass cleanly.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` must produce zero warnings.
  - `cargo fmt --all -- --check` must pass.
  - `cargo test --workspace --all-features` must pass.
* **Browser Runtime & Modules**:
  - Module APIs must be decoupled from Gecko and unit-tested in isolation.
  - No `unsafe` without an explicit Architecture Decision Record (ADR).

---

## 🚫 Forbidden AI Behaviors

* **Unsolicited Mass PRs**: Automated bots opening batch PRs without maintainer pre-approval.
* **Privacy Leaks**: Never commit cookies, browser profiles, user data, or tokens.
* **Review Spam**: Posting automated LLM code reviews on PRs unless specifically invited by the repository maintainers.

---

## 🏛️ Code of Conduct Compliance

All interactions generated or facilitated by AI tools must strictly adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). Hostile, entitled, or automated spam responses will lead to immediate repository-level and organization-level blocking.
