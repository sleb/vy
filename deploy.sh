#!/bin/bash

# Vy Web - Google Cloud Run Deployment Script
# This script builds and deploys the vy-web service to Google Cloud Run

set -e  # Exit on any error

# Configuration - Update these values for your project
PROJECT_ID=""
REGION="us-central1"
SERVICE_NAME="vy-web"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if required tools are installed
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v gcloud &> /dev/null; then
        log_error "gcloud CLI is not installed. Please install it first:"
        echo "https://cloud.google.com/sdk/docs/install"
        exit 1
    fi

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install it first:"
        echo "https://docs.docker.com/get-docker/"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# Function to validate project configuration
validate_config() {
    if [ -z "$PROJECT_ID" ]; then
        log_error "PROJECT_ID is not set. Please edit this script and set your Google Cloud Project ID."
        exit 1
    fi

    log_info "Using project: $PROJECT_ID"
    log_info "Using region: $REGION"
    log_info "Service name: $SERVICE_NAME"
}

# Function to setup Google Cloud
setup_gcloud() {
    log_info "Setting up Google Cloud configuration..."

    # Set the project
    gcloud config set project "$PROJECT_ID"

    # Enable required APIs
    log_info "Enabling required Google Cloud APIs..."
    gcloud services enable cloudbuild.googleapis.com
    gcloud services enable run.googleapis.com
    gcloud services enable containerregistry.googleapis.com

    # Configure Docker to use gcloud as a credential helper
    gcloud auth configure-docker

    log_success "Google Cloud setup completed"
}

# Function to create secrets (if they don't exist)
create_secrets() {
    log_info "Checking for required secrets..."

    # Check if secrets exist, if not, prompt user to create them
    if ! gcloud secrets describe vy-secrets --quiet &>/dev/null; then
        log_warning "Secret 'vy-secrets' does not exist."
        log_info "You need to create the following secrets manually:"
        echo ""
        echo "1. Create the secret:"
        echo "   gcloud secrets create vy-secrets"
        echo ""
        echo "2. Add the required secret versions:"
        echo "   echo 'your-openai-api-key' | gcloud secrets versions add vy-secrets --data-file=- --secret-id=llm-api-key"
        echo "   echo 'your-google-api-key' | gcloud secrets versions add vy-secrets --data-file=- --secret-id=google-api-key"
        echo "   echo 'your-search-engine-id' | gcloud secrets versions add vy-secrets --data-file=- --secret-id=google-search-engine-id"
        echo ""
        echo "Or use the secrets setup script: ./setup-secrets.sh"
        echo ""
        read -p "Press Enter after creating the secrets, or Ctrl+C to exit..."
    else
        log_success "Secret 'vy-secrets' exists"
    fi
}

# Function to build Docker image
build_image() {
    log_info "Building Docker image..."

    # Build the image
    docker build -t "$IMAGE_NAME:latest" .

    log_success "Docker image built successfully"
}

# Function to push image to Google Container Registry
push_image() {
    log_info "Pushing image to Google Container Registry..."

    docker push "$IMAGE_NAME:latest"

    log_success "Image pushed to registry"
}

# Function to deploy to Cloud Run
deploy_service() {
    log_info "Deploying to Cloud Run..."

    # Deploy using gcloud run deploy
    gcloud run deploy "$SERVICE_NAME" \
        --image="$IMAGE_NAME:latest" \
        --platform=managed \
        --region="$REGION" \
        --allow-unauthenticated \
        --memory=512Mi \
        --cpu=1 \
        --concurrency=100 \
        --max-instances=10 \
        --min-instances=0 \
        --timeout=300 \
        --set-env-vars="VY_LLM_MODEL_ID=gpt-4o-mini,VY_MEMORY_MODEL_ID=gpt-4o-mini,VY_MEMORY_SIMILARITY_MODEL_ID=gpt-4o-mini,VY_DEFAULT_CHAT_MODE=web,VY_QDRANT_URL=http://localhost:6333,VY_COLLECTION_NAME=vy_memories,VY_EMBEDDING_MODEL=text-embedding-3-small" \
        --set-secrets="VY_LLM_API_KEY=vy-secrets:llm-api-key:latest,VY_GOOGLE_API_KEY=vy-secrets:google-api-key:latest,VY_GOOGLE_SEARCH_ENGINE_ID=vy-secrets:google-search-engine-id:latest"

    log_success "Service deployed successfully"

    # Get the service URL
    SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" --region="$REGION" --format="value(status.url)")
    log_success "Service is available at: $SERVICE_URL"
    log_info "Health check: $SERVICE_URL/health"
    log_info "Chat API: $SERVICE_URL/api/chat"
}

# Function to show service logs
show_logs() {
    log_info "Recent service logs:"
    gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=$SERVICE_NAME" --limit=20 --format="table(timestamp,severity,textPayload)"
}

# Main deployment function
main() {
    echo ""
    log_info "Starting Vy Web deployment to Google Cloud Run"
    echo ""

    check_prerequisites
    validate_config
    setup_gcloud
    create_secrets
    build_image
    push_image
    deploy_service

    echo ""
    log_success "Deployment completed successfully!"
    echo ""
    log_info "Useful commands:"
    echo "  View logs:    gcloud logs tail --log-filter='resource.type=cloud_run_revision AND resource.labels.service_name=$SERVICE_NAME'"
    echo "  Update env:   gcloud run services update $SERVICE_NAME --region=$REGION --set-env-vars=KEY=VALUE"
    echo "  Scale down:   gcloud run services update $SERVICE_NAME --region=$REGION --min-instances=0"
    echo "  Delete:       gcloud run services delete $SERVICE_NAME --region=$REGION"
    echo ""
}

# Command line argument handling
case "${1:-}" in
    "build")
        check_prerequisites
        validate_config
        build_image
        ;;
    "push")
        check_prerequisites
        validate_config
        push_image
        ;;
    "deploy")
        check_prerequisites
        validate_config
        deploy_service
        ;;
    "logs")
        validate_config
        show_logs
        ;;
    "")
        main
        ;;
    *)
        echo "Usage: $0 [build|push|deploy|logs]"
        echo ""
        echo "Commands:"
        echo "  (no args)  - Full deployment pipeline"
        echo "  build      - Build Docker image only"
        echo "  push       - Push image to registry only"
        echo "  deploy     - Deploy to Cloud Run only"
        echo "  logs       - Show recent service logs"
        exit 1
        ;;
esac
