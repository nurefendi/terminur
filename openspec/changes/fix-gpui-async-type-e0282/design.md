# Design

The GPUI Context<RustTerm>::spawn closure receives a WeakEntity<RustTerm> and AsyncApp. Annotate both closure parameters explicitly. Keep the existing timer and cx.notify logic. No dependency additions or SonarQube integration are required for this narrow fix.
