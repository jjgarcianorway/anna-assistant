#!/usr/bin/env python3
"""
Anna QA Test Harness - Arch Linux Questions

Runs Anna against test questions and compares output to golden reference answers.
Produces machine-readable results with PASS/PARTIAL/FAIL verdicts.

Usage:
    ./run_qa_suite.py --all              # Run all questions
    ./run_qa_suite.py --id arch-001      # Run specific question
    ./run_qa_suite.py --category networking  # Run category
    ./run_qa_suite.py --count 5          # Run first N questions
"""

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple

# Paths
SCRIPT_DIR = Path(__file__).parent
QUESTIONS_FILE = SCRIPT_DIR / "questions_archlinux.jsonl"
GOLDEN_DIR = SCRIPT_DIR / "golden"
RESULTS_DIR = SCRIPT_DIR / "results"
ANNACTL_PATH = Path.home() / "anna-assistant" / "target" / "release" / "annactl"

# Ensure results directory exists
RESULTS_DIR.mkdir(exist_ok=True)


def load_questions() -> List[Dict]:
    """Load all questions from JSONL file."""
    questions = []
    with open(QUESTIONS_FILE, 'r') as f:
        for line in f:
            line = line.strip()
            if line:
                questions.append(json.loads(line))
    return questions


def load_golden_answer(question_id: str) -> Dict:
    """Load golden answer for a question."""
    golden_file = GOLDEN_DIR / f"{question_id}_golden.json"
    if not golden_file.exists():
        return None
    with open(golden_file, 'r') as f:
        return json.load(f)


def run_annactl(question: str, question_id: str) -> Tuple[str, int, str]:
    """
    Run annactl with a question and capture output.

    Returns:
        (stdout, return_code, stderr)
    """
    output_file = RESULTS_DIR / f"{question_id}_anna.txt"

    try:
        # Run annactl with timeout
        result = subprocess.run(
            [str(ANNACTL_PATH), question],
            capture_output=True,
            text=True,
            timeout=60  # 60 second timeout
        )

        # Save raw output
        with open(output_file, 'w') as f:
            f.write(f"# Question: {question}\n")
            f.write(f"# Question ID: {question_id}\n")
            f.write(f"# Timestamp: {datetime.now().isoformat()}\n")
            f.write(f"# Return code: {result.returncode}\n")
            f.write("\n--- STDOUT ---\n")
            f.write(result.stdout)
            f.write("\n--- STDERR ---\n")
            f.write(result.stderr)

        return result.stdout, result.returncode, result.stderr

    except subprocess.TimeoutExpired:
        error_msg = f"TIMEOUT: annactl did not respond within 60 seconds"
        with open(output_file, 'w') as f:
            f.write(f"# Question: {question}\n")
            f.write(f"# ERROR: {error_msg}\n")
        return "", -1, error_msg

    except Exception as e:
        error_msg = f"ERROR: {str(e)}"
        with open(output_file, 'w') as f:
            f.write(f"# Question: {question}\n")
            f.write(f"# ERROR: {error_msg}\n")
        return "", -1, error_msg


def evaluate_answer(anna_output: str, golden: Dict, question_id: str) -> Dict:
    """
    Compare Anna's output against golden answer.

    Returns verdict dict with:
    - verdict: PASS/PARTIAL/FAIL
    - score: 0.0 to 1.0
    - issues: List of problems found
    - missing_commands: Commands that should be mentioned
    - missing_concepts: Concepts that should be covered
    """
    if not golden:
        return {
            "verdict": "SKIP",
            "score": 0.0,
            "issues": ["No golden answer available yet"],
            "missing_commands": [],
            "missing_concepts": []
        }

    anna_lower = anna_output.lower()
    issues = []
    missing_commands = []
    missing_concepts = []

    golden_answer = golden.get("golden_answer", {})

    # Check for required commands
    required_commands = golden_answer.get("required_commands", [])
    for cmd in required_commands:
        # Check if command appears in output (handle variations like sudo, backticks, etc.)
        cmd_base = cmd.replace("sudo ", "").strip()
        if cmd_base not in anna_lower and cmd not in anna_lower:
            missing_commands.append(cmd)
            issues.append(f"Missing required command: {cmd}")

    # Check for required files
    required_files = golden_answer.get("required_files", [])
    for file_path in required_files:
        if file_path not in anna_lower:
            issues.append(f"Missing required file: {file_path}")

    # Check for required concepts
    required_concepts = golden_answer.get("required_concepts", [])
    for concept in required_concepts:
        if concept.lower() not in anna_lower:
            missing_concepts.append(concept)
            issues.append(f"Missing key concept: {concept}")

    # Check for warnings (security/safety)
    warnings = golden_answer.get("warnings", [])
    warning_keywords = ["backup", "warning", "careful", "caution", "risk", "danger", "critical"]
    has_warnings = any(keyword in anna_lower for keyword in warning_keywords)
    if warnings and not has_warnings:
        issues.append("Missing safety warnings (backup, risks, etc.)")

    # Check output is not empty or error
    if not anna_output or len(anna_output.strip()) < 50:
        issues.append("Output too short or empty")
        return {
            "verdict": "FAIL",
            "score": 0.0,
            "issues": issues,
            "missing_commands": missing_commands,
            "missing_concepts": missing_concepts
        }

    # Check for obvious error patterns
    error_patterns = [
        "error:",
        "failed:",
        "cannot",
        "unknown command",
        "not found",
        "planner error",
        "llm call failed"
    ]
    for pattern in error_patterns:
        if pattern in anna_lower:
            issues.append(f"Output contains error pattern: '{pattern}'")

    # Calculate score
    total_checks = len(required_commands) + len(required_files) + len(required_concepts) + (1 if warnings else 0)
    failed_checks = len(missing_commands) + len([i for i in issues if "Missing required file" in i]) + len(missing_concepts) + (1 if warnings and not has_warnings else 0)

    if total_checks == 0:
        score = 0.5  # No golden criteria defined yet
    else:
        score = 1.0 - (failed_checks / total_checks)

    # Determine verdict
    if score >= 0.9 and len(issues) == 0:
        verdict = "PASS"
    elif score >= 0.6 and len([i for i in issues if "error pattern" in i.lower()]) == 0:
        verdict = "PARTIAL"
    else:
        verdict = "FAIL"

    return {
        "verdict": verdict,
        "score": round(score, 2),
        "issues": issues,
        "missing_commands": missing_commands,
        "missing_concepts": missing_concepts
    }


def run_test_suite(question_ids: List[str] = None, category: str = None, count: int = None):
    """Run the test suite on specified questions."""

    # Load all questions
    all_questions = load_questions()

    # Filter questions
    if question_ids:
        questions = [q for q in all_questions if q["id"] in question_ids]
    elif category:
        questions = [q for q in all_questions if q["category"] == category]
    elif count:
        questions = all_questions[:count]
    else:
        questions = all_questions

    if not questions:
        print("No questions to run!")
        return

    print(f"Running {len(questions)} questions...")
    print(f"Anna version: {ANNACTL_PATH}")
    print(f"Results will be saved to: {RESULTS_DIR}")
    print("-" * 80)

    results = {
        "run_timestamp": datetime.now().isoformat(),
        "anna_version": "5.7.0-beta.149",  # TODO: Get from annactl -V
        "total_questions": len(questions),
        "pass": 0,
        "partial": 0,
        "fail": 0,
        "skip": 0,
        "results": []
    }

    for i, question in enumerate(questions, 1):
        qid = question["id"]
        qtext = question["question"]

        print(f"\n[{i}/{len(questions)}] {qid}: {qtext}")

        # Load golden answer
        golden = load_golden_answer(qid)
        if not golden:
            print(f"  ⚠️  SKIP: No golden answer available")
            results["skip"] += 1
            results["results"].append({
                "id": qid,
                "question": qtext,
                "verdict": "SKIP",
                "score": 0.0,
                "issues": ["No golden answer available"],
                "anna_output_file": f"results/{qid}_anna.txt"
            })
            continue

        # Run annactl
        print(f"  Running annactl...")
        anna_output, return_code, stderr = run_annactl(qtext, qid)

        # Evaluate
        print(f"  Evaluating...")
        evaluation = evaluate_answer(anna_output, golden, qid)

        # Update counters
        verdict = evaluation["verdict"]
        if verdict == "PASS":
            results["pass"] += 1
            print(f"  ✅ PASS (score: {evaluation['score']})")
        elif verdict == "PARTIAL":
            results["partial"] += 1
            print(f"  ⚠️  PARTIAL (score: {evaluation['score']})")
        elif verdict == "FAIL":
            results["fail"] += 1
            print(f"  ❌ FAIL (score: {evaluation['score']})")
        else:
            results["skip"] += 1
            print(f"  ⏭️  SKIP")

        # Show issues
        if evaluation["issues"]:
            for issue in evaluation["issues"][:3]:  # Show first 3
                print(f"     - {issue}")
            if len(evaluation["issues"]) > 3:
                print(f"     ... and {len(evaluation['issues']) - 3} more issues")

        # Store result
        results["results"].append({
            "id": qid,
            "question": qtext,
            "category": question.get("category", "unknown"),
            "verdict": verdict,
            "score": evaluation["score"],
            "issues": evaluation["issues"],
            "missing_commands": evaluation.get("missing_commands", []),
            "missing_concepts": evaluation.get("missing_concepts", []),
            "anna_output_file": f"results/{qid}_anna.txt",
            "golden_file": f"golden/{qid}_golden.json"
        })

    # Save results
    summary_file = RESULTS_DIR / "summary.json"
    with open(summary_file, 'w') as f:
        json.dump(results, f, indent=2)

    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Total: {results['total_questions']}")
    print(f"✅ PASS: {results['pass']}")
    print(f"⚠️  PARTIAL: {results['partial']}")
    print(f"❌ FAIL: {results['fail']}")
    print(f"⏭️  SKIP: {results['skip']}")
    print(f"\nPass rate: {results['pass'] / max(1, results['total_questions'] - results['skip']) * 100:.1f}%")
    print(f"Results saved to: {summary_file}")


def main():
    parser = argparse.ArgumentParser(description="Anna QA Test Harness")
    parser.add_argument("--all", action="store_true", help="Run all questions")
    parser.add_argument("--id", help="Run specific question ID (e.g., arch-001)")
    parser.add_argument("--category", help="Run all questions in category")
    parser.add_argument("--count", type=int, help="Run first N questions")

    args = parser.parse_args()

    # Validate annactl exists
    if not ANNACTL_PATH.exists():
        print(f"ERROR: annactl not found at {ANNACTL_PATH}")
        print("Please build annactl first: cargo build --release --bin annactl")
        sys.exit(1)

    # Run tests
    if args.all:
        run_test_suite()
    elif args.id:
        run_test_suite(question_ids=[args.id])
    elif args.category:
        run_test_suite(category=args.category)
    elif args.count:
        run_test_suite(count=args.count)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
