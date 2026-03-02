# Deploy Sidekickz to GCP Free Tier (e2-micro in us-central1)

Write-Host "Deploying Sidekickz to Google Cloud Compute Engine..."

# Load .env variables manually in PowerShell if they exist
if (Test-Path .env) {
    Get-Content .env | Foreach-Object {
        $name, $value = $_.Split('=', 2)
        Set-Item -Path Env:\$name -Value $value
    }
}

# The container image must be hosted on a registry.
$CONTAINER_IMAGE = "ghcr.io/clarkernest56/zeroclaw:latest"

gcloud compute instances create-with-container sidekickz-free-tier-vm `
    --project="YOUR_GCP_PROJECT_ID" `
    --zone="us-central1-a" `
    --machine-type="e2-micro" `
    --network-interface="network-tier=PREMIUM,subnet=default" `
    --boot-disk-size="30GB" `
    --boot-disk-type="pd-standard" `
    --container-image="$CONTAINER_IMAGE" `
    --container-env="DAILY_API_KEY=$env:DAILY_API_KEY,SIGNALWIRE_PROJECT=$env:SIGNALWIRE_PROJECT,SIGNALWIRE_TOKEN=$env:SIGNALWIRE_TOKEN,SIGNALWIRE_SPACE=$env:SIGNALWIRE_SPACE" `
    --tags="http-server,https-server"

Write-Host "Retrieving Public IP Address..."
gcloud compute instances describe sidekickz-free-tier-vm `
    --project="YOUR_GCP_PROJECT_ID" `
    --zone="us-central1-a" `
    --format="get(networkInterfaces[0].accessConfigs[0].natIP)"
