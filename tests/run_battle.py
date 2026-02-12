#!/usr/bin/env python3
"""
Anna vs Claude: 100 Question Battle Test
Tracks accuracy AND speed for each question
"""

import subprocess
import time
import re
from datetime import datetime

# Colors
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    CYAN = '\033[0;36m'
    NC = '\033[0m'

def load_questions():
    """Load questions from test file"""
    questions = []
    current_category = ""

    with open('tests/anna_vs_claude_100.txt', 'r') as f:
        for line in f:
            line = line.strip()

            # Skip empty lines and main header
            if not line or line.startswith('#'):
                continue

            # Detect category headers
            if line.startswith('##'):
                current_category = line.replace('##', '').split('-')[0].strip()
                continue

            # Parse question (format: "1. Question text")
            match = re.match(r'^(\d+)\.\s*(.+)$', line)
            if match:
                q_id = int(match.group(1))
                q_text = match.group(2)
                questions.append({
                    'id': q_id,
                    'category': current_category,
                    'text': q_text
                })

    return questions

def ask_anna(question):
    """Ask Anna a question and measure response time"""
    start = time.time()

    try:
        result = subprocess.run(
            ['./target/release/annactl', question],
            capture_output=True,
            text=True,
            timeout=120  # 2 minute timeout per question
        )

        elapsed_ms = int((time.time() - start) * 1000)

        return {
            'success': result.returncode == 0,
            'response': result.stdout,
            'error': result.stderr,
            'time_ms': elapsed_ms
        }
    except subprocess.TimeoutExpired:
        elapsed_ms = int((time.time() - start) * 1000)
        return {
            'success': False,
            'response': '',
            'error': 'TIMEOUT',
            'time_ms': elapsed_ms
        }
    except Exception as e:
        elapsed_ms = int((time.time() - start) * 1000)
        return {
            'success': False,
            'response': '',
            'error': str(e),
            'time_ms': elapsed_ms
        }

def evaluate_response(response, error):
    """Determine if Anna's response is successful"""
    if not response or error:
        return False

    response_lower = response.lower()

    # Signs of failure
    if any(word in response_lower for word in ['error:', 'failed', 'timeout', 'could not']):
        return False

    # Signs of success
    return len(response) > 30  # At least some meaningful content

def main():
    print("🥊 ANNA VS CLAUDE: 100 QUESTION BATTLE TEST 🥊")
    print()
    print(f"Started at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("Expected duration: 30-60 minutes")
    print()
    print("="*80)
    print()

    questions = load_questions()
    print(f"Loaded {len(questions)} questions")
    print()

    results = []
    total_time = 0
    successes = 0
    failures = 0
    current_category = ""

    with open('tests/battle_results.txt', 'w') as f:
        f.write("# ANNA VS CLAUDE: BATTLE TEST RESULTS\n")
        f.write(f"# Generated: {datetime.now()}\n\n")
        f.write("="*80 + "\n\n")

        for q in questions:
            # Print category header
            if q['category'] != current_category:
                current_category = q['category']
                header = f"\n{'='*80}\n📁 CATEGORY: {current_category}\n{'='*80}\n"
                print(Colors.CYAN + header + Colors.NC)
                f.write(header + "\n")

            # Ask Anna
            print(f"Q{q['id']:3d}: {q['text'][:70]:<70} ", end='', flush=True)

            result = ask_anna(q['text'])
            total_time += result['time_ms']

            # Evaluate
            success = evaluate_response(result['response'], result['error'])

            if success:
                successes += 1
                status = f"{Colors.GREEN}✓{Colors.NC} ({result['time_ms']}ms)"
                f.write(f"Q{q['id']}: {q['text']} - SUCCESS ({result['time_ms']}ms)\n")
            else:
                failures += 1
                status = f"{Colors.RED}✗{Colors.NC} ({result['time_ms']}ms)"
                f.write(f"Q{q['id']}: {q['text']} - FAILED ({result['time_ms']}ms)\n")

            print(status)

            results.append({
                'question': q,
                'result': result,
                'success': success
            })

            # Small delay
            time.sleep(0.1)

        # Final statistics
        total = len(questions)
        success_rate = (successes / total * 100) if total > 0 else 0
        avg_time = (total_time / total) if total > 0 else 0
        total_seconds = total_time / 1000

        summary = f"""
{'='*80}
📊 FINAL RESULTS
{'='*80}

Total Questions:    {total}
Successful:         {successes} ({success_rate:.1f}%)
Failed:             {failures} ({(failures/total*100):.1f}%)
Average Time:       {avg_time:.0f} ms
Total Time:         {total_seconds:.2f} seconds

"""
        print(summary)
        f.write(summary)

        # Verdict
        if success_rate >= 90:
            verdict = f"{Colors.GREEN}🏆 VERDICT: Anna is EXCELLENT!{Colors.NC}"
        elif success_rate >= 75:
            verdict = f"{Colors.GREEN}✅ VERDICT: Anna is GOOD!{Colors.NC}"
        elif success_rate >= 50:
            verdict = f"{Colors.YELLOW}⚠️  VERDICT: Anna is DECENT - needs improvement{Colors.NC}"
        else:
            verdict = f"{Colors.RED}❌ VERDICT: Anna needs serious work{Colors.NC}"

        print(verdict)
        f.write(verdict.replace(Colors.GREEN, '').replace(Colors.YELLOW, '').replace(Colors.RED, '').replace(Colors.NC, '') + "\n")

    print("\n✨ Test complete! Results saved to tests/battle_results.txt")

if __name__ == '__main__':
    main()
