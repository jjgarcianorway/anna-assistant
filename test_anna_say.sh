#!/usr/bin/env bash
# Quick demo of the new conversational messaging

# Source the anna_common library
source scripts/anna_common.sh

echo ""
echo "=== Anna's Conversational Messaging Demo ==="
echo ""

# Show greeting ceremony
anna_box narrative \
    "Hi! I'm Anna. I'll take care of your setup." \
    "I'll explain each step as we go. Ready?"

echo ""

# Show different message types
anna_info "Checking your environment now..."
sleep 0.5

anna_narrative "Let me see if everything you need is already installed."
sleep 0.5

anna_ok "Found all dependencies!"
sleep 0.5

anna_warn "I need administrator rights to set up my service. May I?"
sleep 0.5

anna_ok "All done - no errors, no drama."
sleep 0.5

# Show completion ceremony
echo ""
anna_box ok \
    "All done! I've checked everything twice." \
    "You can talk to me anytime using 'annactl'." \
    "Let's see what we can build together."

echo ""
echo "=== Demo Complete ==="
echo ""
echo "This is how Anna will communicate throughout installation and operations."
echo "Try: ./target/release/annactl doctor check"
echo ""
