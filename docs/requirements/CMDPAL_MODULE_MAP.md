# CmdPal — Suggested Module and File Map

This is a starting blueprint, not a demand to create empty folders. The selected language/UI toolkit may adjust naming, but preserve the ownership boundaries.

```text
CmdPal/
├── README.md
├── LICENSE
├── CHANGELOG.md
├── docs/
│   ├── ARCHITECTURE.md
│   ├── FEATURE_CATALOGUE.md
│   ├── PERFORMANCE.md
│   ├── SECURITY.md
│   └── DECISIONS/
│       └── DECISION_LOG.md
├── .github/
│   └── workflows/
│       ├── ci-windows.yml
│       └── release-windows.yml
├── assets/
│   └── app-icon/                 # original CmdPal icon and source asset
├── src/
│   ├── main.*                    # tiny entry point; delegates only
│   ├── bootstrap/
│   │   ├── bootstrap.*           # composition root / lifecycle
│   │   ├── feature_registry.*    # single module registration point
│   │   └── app_config.*          # startup configuration wiring
│   ├── core/
│   │   ├── contracts/            # stable cross-module interfaces
│   │   ├── settings/             # canonical settings model/persistence API
│   │   ├── logging/              # low-overhead structured logging
│   │   └── common/               # only genuinely shared primitives
│   ├── platform/
│   │   └── windows/
│   │       ├── hotkey/
│   │       ├── shell_launch/
│   │       ├── app_discovery/
│   │       ├── filesystem/
│   │       ├── startup/
│   │       ├── monitor_dpi/
│   │       └── windowing/
│   ├── design_system/            # small shared visual primitives
│   └── features/
│       ├── palette/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── suggestions/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── app_search/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── commands/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── web_bangs/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── file_search/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── indexing/
│       │   ├── ui/
│       │   └── tests/
│       ├── quick_folders/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── logic/
│       │   ├── ui/
│       │   └── tests/
│       ├── notes/
│       │   ├── README.md
│       │   ├── api/
│       │   ├── markdown_model/
│       │   ├── persistence/
│       │   ├── ui/
│       │   └── tests/
│       └── settings/
│           ├── README.md
│           ├── api/
│           ├── logic/
│           ├── ui/
│           └── tests/
├── tests/
│   ├── integration/
│   └── smoke/
└── packaging/
    ├── windows/
    └── installer/                # only if installer is chosen
```

## Feature ownership

| Module | Owns | Must not own |
|---|---|---|
| palette | palette lifecycle UI, query input, hotkey interaction contract, focus/placement state | app ranking, file traversal, note parsing |
| suggestions | categories, result aggregation/ranking, exclusion filtering | actual app launching or indexing engine |
| app_search | installed app discovery/cache/search adapter and launch descriptors | general palette window |
| commands | command definitions, user command editor, validation/execution policy | unsafe shell-building in UI |
| web_bangs | alias parsing, chips, URL construction/validation, bookmark management, default-browser handoff | web rendering or embedded browser processes |
| file_search | root configuration, indexer, search, exclusions, index status UI | full-drive scan by default or startup blocking |
| quick_folders | quick folder shortcuts and ordering | file-index roots (keep separate settings) |
| notes | Markdown model, editable live formatting surface, note persistence/recovery, toolbar | palette-wide window lifecycle |
| settings | settings UI, search integration, schema validation and safe persistence | duplicate independent config sources |

## Integration rule

The bootstrap/feature registry is the single obvious integration point. Keep module-to-module dependencies on public interfaces only. If the chosen UI stack needs a different folder structure, keep the same ownership boundaries.
