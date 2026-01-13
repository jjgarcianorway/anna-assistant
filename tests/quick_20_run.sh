#!/bin/bash
echo "Quick 20 Question Test - $(date)"
echo "======================================"

run_test() {
  local num=$1
  local q="$2"
  echo -n "[$num/20] $q... "
  
  start=$(date +%s.%N)
  result=$(timeout 30 ../target/release/annactl "$q" 2>&1)
  exit_code=$?
  end=$(date +%s.%N)
  elapsed=$(echo "$end - $start" | bc)
  
  if [ $exit_code -eq 124 ]; then
    echo "TIMEOUT"
    return 1
  fi
  
  # Check for clarification
  if echo "$result" | grep -qiE "clarif|more specific|could you please|what exactly"; then
    echo "CLARIFIED (${elapsed}s)"
    return 2
  fi
  
  echo "OK (${elapsed}s)"
  return 0
}

# Run tests
run_test 1 "What kernel version am I running?"
run_test 2 "How much RAM do I have?"
run_test 3 "What GPU driver is loaded?"
run_test 4 "What shell am I using?"
run_test 5 "What services failed to start?"
run_test 6 "How many packages are installed?"
run_test 7 "What is my local IP address?"
run_test 8 "Is the firewall enabled?"
run_test 9 "What audio server is running?"
run_test 10 "What is my CPU temperature?"
run_test 11 "What DNS servers are configured?"
run_test 12 "Is sshd enabled?"
run_test 13 "What USB devices are connected?"
run_test 14 "What desktop environment am I using?"
run_test 15 "What is mounted at /home?"
run_test 16 "Are there any orphaned packages?"
run_test 17 "What ports are listening?"
run_test 18 "Is swap enabled?"
run_test 19 "What filesystem is root using?"
run_test 20 "How long has my system been up?"

echo ""
echo "Done!"
