# Desktop foundation: saved host management

Owner approval: “kerjakan audit dan fondasi desktop”; tracked in #8.

The current macOS SSH prototype can add hosts but cannot edit or delete them. This increment completes that local workflow before larger platform/security work. It does not claim Termius parity.

Scope: prefilled edit form, replacement without duplication, confirmed deletion, persistence failure handling, unchanged running sessions. No dependencies, SSH policy, credentials, workflow or permissions changes.

Risks: index-based selection assumes a single UI-owned host list; external concurrent JSON writers remain unsupported. Native GUI acceptance is required. Windows/Linux are planned desktop targets, not verified by macOS CI; Android and iOS are explicit later milestones.

Dependency order after this increment: native Windows/Linux CI and platform assumptions → real-host SSH checks per desktop → reviewed OS credential vault → terminal copy/search and SFTP/forwarding → Android native/device milestone → iOS native/device milestone → optional encrypted sync. Workflow and sensitive credential changes require owner review.
