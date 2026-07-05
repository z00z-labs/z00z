# Changelog

All notable changes to the code-refactoring skill will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [2.0.0] - 2025-10-31

### Added
- **🎯 Path-Based Thresholds** (MAJOR FEATURE)
  - File watcher now uses intelligent path-based thresholds instead of just file extensions
  - Different thresholds for different file types based on their purpose:
    - `page.tsx` files: 300/500/800 lines (educational content/demos allowed)
    - `data/**/*.tsx` files: 250/400/600 lines (mostly static content)
    - `components/**/*.tsx` files: 150/200/300 lines (standard threshold)
    - `lib/**/*.ts` & `utils/**/*.ts`: 100/150/200 lines (strict - should be small)
    - `api/**/*.ts` files: 100/150/250 lines (thin controller preferred)
  - Falls back to extension-based thresholds for files that don't match patterns
  - Thresholds now show their source (path-based vs extension-based) for transparency

### Changed
- **Breaking:** Thresholds are now calculated per-file based on path, not just extension
- Alert messages now include threshold source information `[path-based (reason)]`
- RAG Playground pages now get appropriate 800-line critical threshold (was 300)

### Technical Details
- Added `getThresholds(filePath)` function for intelligent threshold selection
- Path patterns use normalized paths for cross-platform compatibility (Windows & Unix)
- Thresholds include `reason` field explaining why a threshold was chosen
- Cross-platform regex patterns: `/[/\\]data[/\\]/` works on both Windows and Unix

### Benefits
- ✅ **Solves false positives** - Educational pages no longer flagged incorrectly
- ✅ **Context-aware** - Logic files held to stricter standards than UI files
- ✅ **Industry-aligned** - Follows real-world developer mental models
- ✅ **Transparent** - Users see why each threshold was applied
- ✅ **Fast** - Simple pattern matching, no parsing overhead (80% solution, 20% effort)
- ✅ **Extensible** - Easy to add new path patterns for your project

### Migration Guide
**No action required!** The skill automatically detects file types and applies appropriate thresholds. Existing behavior for unmatched files remains unchanged (falls back to extension-based thresholds from v1.x).

### Example Results

**Before v2.0 (Extension-based only):**
- `page.tsx` (1,246 lines) → CRITICAL ❌ (>300 lines) - False positive!
- `utils/helper.ts` (250 lines) → ALERT ⚠️ (>200 lines)

**After v2.0 (Path-based + Extension-based):**
- `page.tsx` (1,246 lines) → ALERT ⚠️ (>500, <800 for pages) ✅ More reasonable!
- `utils/helper.ts` (250 lines) → CRITICAL 🛑 (>200 for utilities) ✅ Correctly strict!

---

## [1.1.1] - 2025-10-30

### Changed
- **Streamlined slash commands** from 5 to 3 essential commands
  - Removed `/check-refactor-alerts` (redundant - skill auto-checks)
  - Removed `/watcher-status` (redundant - `/start-watcher` shows status)
  - Kept `/start-watcher`, `/stop-watcher`, `/scan-code-size`

### Added
- **COMMANDS_REFERENCE.md** - Comprehensive slash command documentation
- **SKILL_QUALITY_REPORT.md** - Quality assessment by skill-builder (94/100)
- **File watcher section in README.md** - Real-time monitoring documentation
- **Slash commands section in README.md** - User-friendly command guide

### Fixed
- Updated SKILL.md to remove references to deleted commands
- Updated all documentation to reflect streamlined command set

### Quality
- Skill-builder assessment: **94/100 (Grade A - EXCELLENT)**
- Status: **PRODUCTION READY**

---

## [1.1.0] - 2025-10-30

### Added
- **Real-time file watcher system** with background process monitoring
  - `file-watcher.js` - Main watcher daemon
  - `check-alerts.js` - Alert display and management
  - `start-watcher.sh/.bat` - Cross-platform start scripts
  - `stop-watcher.sh/.bat` - Process termination scripts
  - `auto-start-watcher.js` - Session-start hook integration

- **Alert system integrated with chat-visible alert files**
  - Alerts written to `watcher-alerts.json`
  - Automatic alert checking in SKILL.md (critical first step)
  - Real-time notifications when editing oversized files
  - Growth-based alerts (50+ line additions trigger immediate alert)

- **Slash commands** for user control
  - `/start-watcher` - Start background monitoring
  - `/stop-watcher` - Stop background monitoring
  - `/check-refactor-alerts` - Manual alert checking (later removed in v1.1.1)
  - `/watcher-status` - Process status check (later removed in v1.1.1)
  - `/scan-code-size` - One-time codebase scan

- **Auto-start through session-start hooks**
  - Configured through the host environment's startup hook system
  - Silent background activation
  - Duplicate process prevention

- **Documentation for alert system**
  - `ALERT_SYSTEM_FIX.md` - Technical implementation details
  - `AUTOSTART_GUIDE.md` - Setup and configuration guide
  - `WATCHER_README.md` - Watcher system documentation

### Fixed
- Alert notifications now appear through assistant-readable alert files (was silent before v1.1.0)
- File watcher integrates cleanly with coding-assistant workflows
- Alerts properly grouped by severity (critical/alert/warning)

### Technical Details
- File watcher monitors: `.js`, `.jsx`, `.ts`, `.tsx`, `.py` files
- Alert thresholds: 150 (warning), 200 (alert), 300+ (critical) lines
- Reminder intervals: 5min (critical), 15min (alert), 30min (warning)
- Growth threshold: 50+ lines triggers immediate alert
- Cross-platform: Windows (.bat) + Unix/Linux/macOS (.sh) + Node.js

---

## [1.0.0] - 2025-10-30

### Added
- **Initial release** of code-refactoring skill

- **Multi-language support**
  - JavaScript/TypeScript/React patterns
  - Python patterns
  - General refactoring principles

- **Progressive disclosure architecture**
  - SKILL.md (~2,689 tokens)
  - REFERENCE.md (1,754 lines - loaded on demand)
  - FORMS.md (1,058 lines - templates)
  - Token-efficient design (46% under 5k target)

- **Research-backed approach**
  - Based on Martin Fowler's "Refactoring"
  - Kent Beck's "Extreme Programming" patterns
  - 21 code smells catalog
  - Authoritative sources comparison

- **Auto-invoke conditions**
  - File size thresholds (150/200/300 lines)
  - Pattern detection (hooks, data arrays, classes)
  - Language-specific triggers

- **Execution phase**
  - Step-by-step refactoring with user approval
  - Incremental commits for each step
  - Automatic rollback on failure
  - Test-driven approach

- **Supporting resources**
  - `code-smells-catalog.md` - Fowler's 21 code smells
  - `AUTHORITATIVE_SOURCES_COMPARISON.md` - Validation vs industry standards
  - `diagrams/decision-flowchart.md` - 8 Mermaid decision flowcharts
  - `examples/` directory structure (empty - to be filled)

- **Helper scripts**
  - `check-size.sh` - File size checker
  - `analyze-codebase.sh` - Full codebase audit

- **Documentation**
  - README.md - User guide
  - SKILL.md - AI instructions
  - REFERENCE.md - Detailed procedures
  - FORMS.md - Templates and checklists

### Design Principles
- Prevention over cure (proactive monitoring)
- Progressive disclosure (minimal token usage)
- Language-agnostic with language-specific customization
- Research-backed patterns and thresholds
- User control with AI assistance

---

## Versioning Strategy

- **Major version (X.0.0)**: Breaking changes, major feature additions
- **Minor version (1.X.0)**: New features, backward compatible
- **Patch version (1.1.X)**: Bug fixes, documentation updates

---

## Future Enhancements (Roadmap)

### Planned for v1.2.0
- [ ] Add before/after code examples to `resources/examples/`
- [ ] Create QUICK_REFERENCE.md cheat sheet
- [ ] Add metrics dashboard script (`generate-metrics-report.js`)
- [ ] Team notifications (Slack/Teams integration)
- [ ] Refactoring progress tracking over time

### Under Consideration
- [ ] Real-time push notifications (vs. polling)
- [ ] Integration with git hooks (pre-commit checks)
- [ ] Smart alert throttling based on user behavior
- [ ] Additional language support (Java, C#, Go, Rust)
- [ ] Visual dashboard for codebase health

---

## Credits

**Created by:** Madina Gbotoe (https://madinagbotoe.com/)
**License:** Creative Commons Attribution 4.0 International (CC BY 4.0)
**Attribution Required:** Yes - Include author name and link when sharing/modifying
**Original attribution:** Madina Gbotoe (https://madinagbotoe.com/)

---

## Links

- **Documentation:** See README.md in skill directory
- **Quality Report:** See SKILL_QUALITY_REPORT.md (94/100)
