---

description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks are MANDATORY. Every user story MUST include tests written BEFORE the implementation, following the Red-Green-Refactor TDD cycle.

**TDD Cycle (REQUIRED for every implementation task)**:

1. **RED**: Write a failing test that specifies the desired behavior. Run it and confirm it fails for the expected reason.
2. **GREEN**: Write the minimum production code required to make the test pass. Do not add behavior the test does not require.
3. **REFACTOR**: Improve structure, naming, and duplication while keeping all tests green. Re-run the full suite after each refactor step.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume single project - adjust based on plan.md structure

<!-- 
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.
  
  The /speckit.tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  - Entities from data-model.md
  - Endpoints from contracts/
  
  Tasks MUST be organized by user story so each story can be:
  - Implemented independently
  - Tested independently
  - Delivered as an MVP increment
  
  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize [language] project with [framework] dependencies
- [ ] T003 [P] Configure linting and formatting tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

Examples of foundational tasks (adjust based on your project):

- [ ] T004 Setup database schema and migrations framework
- [ ] T005 [P] Implement authentication/authorization framework
- [ ] T006 [P] Setup API routing and middleware structure
- [ ] T007 Create base models/entities that all stories depend on
- [ ] T008 Configure error handling and logging infrastructure
- [ ] T009 Setup environment configuration management

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1 (MANDATORY - RED phase) ⚠️

> **TDD RULE: Write these tests FIRST. Run them and confirm they FAIL for the expected reason before writing any implementation.**

- [ ] T010 [P] [US1] [RED] Contract test for [endpoint] in tests/contract/test_[name].py — must fail before T014/T015
- [ ] T011 [P] [US1] [RED] Integration test for [user journey] in tests/integration/test_[name].py — must fail before T014/T015
- [ ] T011a [P] [US1] [RED] Unit tests for [Entity1] behavior in tests/unit/test_[entity1].py — must fail before T012
- [ ] T011b [P] [US1] [RED] Unit tests for [Entity2] behavior in tests/unit/test_[entity2].py — must fail before T013

### Implementation for User Story 1 (GREEN phase — minimum code to pass failing tests)

- [ ] T012 [P] [US1] [GREEN] Create [Entity1] model in src/models/[entity1].py to pass T011a
- [ ] T013 [P] [US1] [GREEN] Create [Entity2] model in src/models/[entity2].py to pass T011b
- [ ] T014 [US1] [GREEN] Implement [Service] in src/services/[service].py to pass T011 (depends on T012, T013)
- [ ] T015 [US1] [GREEN] Implement [endpoint/feature] in src/[location]/[file].py to pass T010
- [ ] T016 [US1] [GREEN] Add validation and error handling driven by failing edge-case tests
- [ ] T017 [US1] [GREEN] Add logging for user story 1 operations

### Refactor for User Story 1 (REFACTOR phase — all tests must stay green)

- [ ] T017a [US1] [REFACTOR] Remove duplication and clarify naming across US1 files; re-run full test suite
- [ ] T017b [US1] [REFACTOR] Verify coverage ≥ 80% for US1 modules

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2 (MANDATORY - RED phase) ⚠️

> **TDD RULE: Write tests first and confirm they FAIL before implementation.**

- [ ] T018 [P] [US2] [RED] Contract test for [endpoint] in tests/contract/test_[name].py — must fail before T021/T022
- [ ] T019 [P] [US2] [RED] Integration test for [user journey] in tests/integration/test_[name].py — must fail before T021/T022
- [ ] T019a [P] [US2] [RED] Unit tests for [Entity] behavior in tests/unit/test_[entity].py — must fail before T020

### Implementation for User Story 2 (GREEN phase)

- [ ] T020 [P] [US2] [GREEN] Create [Entity] model in src/models/[entity].py to pass T019a
- [ ] T021 [US2] [GREEN] Implement [Service] in src/services/[service].py to pass T019
- [ ] T022 [US2] [GREEN] Implement [endpoint/feature] in src/[location]/[file].py to pass T018
- [ ] T023 [US2] [GREEN] Integrate with User Story 1 components (if needed)

### Refactor for User Story 2 (REFACTOR phase)

- [ ] T023a [US2] [REFACTOR] Remove duplication across US2 files; re-run full test suite
- [ ] T023b [US2] [REFACTOR] Verify coverage ≥ 80% for US2 modules

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3 (MANDATORY - RED phase) ⚠️

> **TDD RULE: Write tests first and confirm they FAIL before implementation.**

- [ ] T024 [P] [US3] [RED] Contract test for [endpoint] in tests/contract/test_[name].py — must fail before T027/T028
- [ ] T025 [P] [US3] [RED] Integration test for [user journey] in tests/integration/test_[name].py — must fail before T027/T028
- [ ] T025a [P] [US3] [RED] Unit tests for [Entity] behavior in tests/unit/test_[entity].py — must fail before T026

### Implementation for User Story 3 (GREEN phase)

- [ ] T026 [P] [US3] [GREEN] Create [Entity] model in src/models/[entity].py to pass T025a
- [ ] T027 [US3] [GREEN] Implement [Service] in src/services/[service].py to pass T025
- [ ] T028 [US3] [GREEN] Implement [endpoint/feature] in src/[location]/[file].py to pass T024

### Refactor for User Story 3 (REFACTOR phase)

- [ ] T028a [US3] [REFACTOR] Remove duplication across US3 files; re-run full test suite
- [ ] T028b [US3] [REFACTOR] Verify coverage ≥ 80% for US3 modules

**Checkpoint**: All user stories should now be independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] TXXX [P] Documentation updates in docs/
- [ ] TXXX [REFACTOR] Code cleanup and refactoring (all tests must stay green)
- [ ] TXXX [REFACTOR] Performance optimization across all stories (re-run full test suite after each change)
- [ ] TXXX [P] [RED] Additional unit tests in tests/unit/ to cover uncovered branches; must fail before filling gaps
- [ ] TXXX [GREEN] Fill coverage gaps surfaced by the unit tests above
- [ ] TXXX Security hardening (driven by failing security tests)
- [ ] TXXX Run quickstart.md validation
- [ ] TXXX Verify overall test coverage ≥ 80% and all tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable

### Within Each User Story (TDD Red-Green-Refactor is MANDATORY)

- RED: Tests MUST be written and confirmed FAILING before any implementation task in that story starts
- GREEN: Write the minimum implementation to turn each failing test green; do not add untested behavior
- REFACTOR: Clean up duplication and structure with the full test suite green; re-run tests after every refactor step
- Models before services, services before endpoints, core implementation before integration
- A story is complete only when: all its tests pass, coverage ≥ 80%, and the refactor phase has been executed

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# RED phase — launch all failing tests for User Story 1 together:
Task: "[RED] Contract test for [endpoint] in tests/contract/test_[name].py"
Task: "[RED] Integration test for [user journey] in tests/integration/test_[name].py"
Task: "[RED] Unit tests for [Entity1] in tests/unit/test_[entity1].py"

# Confirm all tests fail, then start GREEN phase in parallel where files are independent:
Task: "[GREEN] Create [Entity1] model in src/models/[entity1].py"
Task: "[GREEN] Create [Entity2] model in src/models/[entity2].py"

# REFACTOR phase — run sequentially with the full suite green after each step.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [RED] / [GREEN] / [REFACTOR] labels are MANDATORY on every implementation-side task so the TDD phase is explicit
- Tests are MANDATORY: every production behavior must be introduced by a previously failing test
- Each user story should be independently completable and testable
- Verify tests fail (and fail for the expected reason) before writing implementation
- After GREEN, always run the REFACTOR step and re-run the full test suite
- Commit after each task or logical group; prefer one commit per RED → GREEN → REFACTOR cycle
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence, writing implementation before a failing test exists
