#!/usr/bin/env bash

# Define the cleanup function that runs when the script exits
cleanup() {
  echo -e "\n[!] Caught exit signal. Stopping Docker containers..."

  docker compose down

  echo "[*] Pruning unused Docker resources..."

  # Remove stopped containers, unused networks, and dangling images
  docker system prune -f

  # Remove build cache (this is often the biggest space hog)
  docker builder prune -af

  # OPTIONAL (more aggressive):
  # Remove unused volumes (WARNING: can delete DB/data if not used by running containers)
  # docker volume prune -f

  echo "[✔] Cleanup complete."
  exit 0
}

# Trap SIGINT (Ctrl+C) and SIGTERM
trap cleanup SIGINT SIGTERM EXIT

echo "[*] Building and starting Docker containers in the background..."
docker compose up --build -d

# Check if the build/start was successful before tailing logs
if [ $? -ne 0 ]; then
  echo "[X] Docker compose failed to start. Exiting."
  exit 1
fi

echo "[*] Attaching to logs. Press Ctrl+C to stop and tear down."
docker compose logs -f

# Fallback (usually not needed because of EXIT trap)
cleanup
