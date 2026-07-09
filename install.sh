#!/bin/bash

# Sample banner
cat << "EOF"
▗▄▄▖  ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▄▖▗▄▄▖      ▗▄▖  ▗▄▄▖▗▄▄▄▖▗▖  ▗▖▗▄▄▄▖
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌  █  ▐▌   ▐▌ ▐▌    ▐▌ ▐▌▐▌   ▐▌   ▐▛▚▖▐▌  █  
▐▛▀▚▖▐▌ ▐▌▐▌ ▐▌  █  ▐▛▀▀▘▐▛▀▚▖    ▐▛▀▜▌▐▌▝▜▌▐▛▀▀▘▐▌ ▝▜▌  █  
▐▌ ▐▌▝▚▄▞▘▝▚▄▞▘  █  ▐▙▄▄▖▐▌ ▐▌    ▐▌ ▐▌▝▚▄▞▘▐▙▄▄▖▐▌  ▐▌  █  
                 By Vihas Methnula :)                           
EOF

echo ""
echo "Starting installation..."
if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo not found"
  echo "Please install cargo before continuing....."
  exit 1
fi

echo "Cloning the repository..."
git clone https://github.com/VihasMethnula/RouterAgent.git
cd RouterAgent
if cargo install --path .; then
  echo "Installation completed successfully."
  echo "Use command \"Router\" after connecting to the router to start the agent."
else
  echo "Installation failed."
  exit 1
fi

