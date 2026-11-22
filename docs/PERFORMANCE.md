# Performance - methodology (no measured claims yet)
Targets are set only after a baseline exists. Nothing here is a measurement unless labelled.
| Metric | How to measure | Where recorded |
|---|---|---|
| Startup to message loop | `CmdPal.exe --smoke` writes `startup_ms` (process start to loop) | CI job summary (runner hardware, not the 4 GB / i3 / HDD target) |
| Peak memory after start | same run: `peak_working_set_kb` | CI job summary |
| Cold/warm palette appearance | manual: hotkey press to first paint, 10 runs after reboot / repeated, on a low-end PC | to be added |
| Idle CPU / RAM | Task Manager / `Get-Process` after 5 min idle, file indexing off and on | to be added |
| Query latency | time `build_rows` for 1-3 character queries with a 100k-entry index (add a benchmark test) | to be added |
| Indexing overhead | CPU and disk activity during a first scan of a large root (Resource Monitor) | to be added |
| Package size | size of the ZIP and exe in the CI artifact | CI summary shows exe size |
Design choices intended to keep these low: lazy window/editor creation, background threads at low CPU+I/O priority, bounded caches and index, no timers while idle apart from the indexer's long wait and a 1 s timer only while the settings window is open.
