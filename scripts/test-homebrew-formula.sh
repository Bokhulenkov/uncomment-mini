#!/bin/bash

set -e

echo "Testing Homebrew formula locally..."

if ! brew tap | grep -q "goldziher/tap"; then
  echo "Adding local tap..."
  brew tap goldziher/tap "$(pwd)/homebrew-tap"
else
  echo "Tap already exists, updating..."
  brew untap goldziher/tap
  brew tap goldziher/tap "$(pwd)/homebrew-tap"
fi

echo "Installing uncomment from local tap..."
brew install --verbose goldziher/tap/uncomment

echo "Testing uncomment command..."
uncomment --version

cat >/tmp/test_uncomment.c <<'EOF'
// This is a comment
int hello(void) {
    return 1; // Inline comment
}
EOF

echo "Running uncomment on test file..."
uncomment --dry-run /tmp/test_uncomment.c

echo "✅ Homebrew formula test completed successfully!"

echo "Cleaning up..."
brew uninstall uncomment
brew untap goldziher/tap
