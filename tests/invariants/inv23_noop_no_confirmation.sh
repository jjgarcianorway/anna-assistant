#!/bin/bash
# Invariant 23: NOOP plans must not trigger confirmation prompts
# When preflight determines no changes are needed, the system must emit
# a terminal response without entering confirmation flow.

set -e

echo "Testing Invariant 23: NOOP plans skip confirmation"

# Check that handlers.rs has the NOOP short-circuit
if ! grep -q "NOOP short-circuit" crates/annad/src/server/streaming/handlers.rs; then
    echo "FAIL: NOOP short-circuit comment not found in handlers.rs"
    exit 1
fi

# Check that NOOP path sets needs_clarification: false
if ! grep -A20 "!plan.changes_needed" crates/annad/src/server/streaming/handlers.rs | grep -q "needs_clarification: false"; then
    echo "FAIL: NOOP path does not set needs_clarification: false"
    exit 1
fi

# Check that NOOP path sets clarification_question: None
if ! grep -A20 "!plan.changes_needed" crates/annad/src/server/streaming/handlers.rs | grep -q "clarification_question: None"; then
    echo "FAIL: NOOP path does not set clarification_question: None"
    exit 1
fi

# Check that NOOP path returns Ok(()) before handle_template_plan
if ! grep -A25 "!plan.changes_needed" crates/annad/src/server/streaming/handlers.rs | grep -q "return Ok(())"; then
    echo "FAIL: NOOP path does not return before handle_template_plan"
    exit 1
fi

echo "Invariant 23: PASS"
