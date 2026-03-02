#!/bin/bash
# Deploy Sidekickz to GCP Free Tier (e2-micro in us-central1)

echo "Deploying Sidekickz to Google Cloud Compute Engine..."

# Ensure environment variables are loaded if running locally
if [ -f .env ]; then
  source .env
fi

# The container image must be hosted on a registry (e.g., Docker Hub or Artifact Registry).
# Since local Docker build is unavailable, we assume pushing the image to a registry is done externally,
# or we use the prebuilt public image (though it lacks the sidekickz local edits unless pushed).
# Update CONTAINER_IMAGE to your actual registry image containing the Sidekickz feature.
CONTAINER_IMAGE="ghcr.io/clarkernest56/zeroclaw:latest"

gcloud compute instances create-with-container sidekickz-free-tier-vm \
    --project="YOUR_GCP_PROJECT_ID" \
    --zone="us-central1-a" \
    --machine-type="e2-micro" \
    --network-interface="network-tier=PREMIUM,subnet=default" \
    --boot-disk-size="30GB" \
    --boot-disk-type="pd-standard" \
    --container-image="$CONTAINER_IMAGE" \
    --container-env="DAILY_API_KEY=${DAILY_API_KEY},SIGNALWIRE_PROJECT=${SIGNALWIRE_PROJECT},SIGNALWIRE_TOKEN=${SIGNALWIRE_TOKEN},SIGNALWIRE_SPACE=${SIGNALWIRE_SPACE}" \
    --tags="http-server,https-server"

echo "Retrieving Public IP Address..."
gcloud compute instances describe sidekickz-free-tier-vm \
    --project="YOUR_GCP_PROJECT_ID" \
    --zone="us-central1-a" \
    --format="get(networkInterfaces[0].accessConfigs[0].natIP)"
