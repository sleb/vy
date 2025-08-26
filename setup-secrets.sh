#!/bin/bash

# Vy Web - Google Cloud Secrets Setup Script
# This script helps you create and manage secrets for the vy-web service

set -e  # Exit on any error

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

# Function to check if gcloud is installed and authenticated
check_gcloud() {
    if ! command -v gcloud &> /dev/null; then
        log_error "gcloud CLI is not installed. Please install it first:"
        echo "https://cloud.google.com/sdk/docs/install"
        exit 1
    fi

    # Check if user is authenticated
    if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | head -n1 &>/dev/null; then
        log_error "You are not authenticated with gcloud. Please run:"
        echo "gcloud auth login"
        exit 1
    fi

    log_success "gcloud is available and you are authenticated"
}

# Function to enable required APIs
enable_apis() {
    log_info "Enabling required Google Cloud APIs..."

    gcloud services enable secretmanager.googleapis.com

    log_success "Secret Manager API enabled"
}

# Function to prompt for secret value securely
prompt_secret() {
    local secret_name="$1"
    local description="$2"

    echo ""
    log_info "$description"
    echo -n "Enter value: "
    read -s secret_value
    echo ""

    if [ -z "$secret_value" ]; then
        log_error "Empty value provided for $secret_name"
        return 1
    fi

    echo "$secret_value"
}

# Function to create or update a secret
create_or_update_secret() {
    local secret_id="$1"
    local secret_value="$2"
    local description="$3"

    # Check if secret exists
    if gcloud secrets describe "$secret_id" --quiet &>/dev/null; then
        log_warning "Secret '$secret_id' already exists. Creating new version..."
        echo "$secret_value" | gcloud secrets versions add "$secret_id" --data-file=-
    else
        log_info "Creating secret '$secret_id'..."
        echo "$secret_value" | gcloud secrets create "$secret_id" --data-file=- --replication-policy="automatic"

        # Add labels for better organization
        gcloud secrets update "$secret_id" --update-labels="app=vy-web,component=api-keys"
    fi

    log_success "Secret '$secret_id' configured successfully"
}

# Function to setup all required secrets
setup_secrets() {
    log_info "Setting up secrets for Vy Web service"
    echo ""
    echo "You will need to provide the following API keys and credentials:"
    echo "1. OpenAI API Key (for LLM functionality)"
    echo "2. Google API Key (for search functionality)"
    echo "3. Google Custom Search Engine ID"
    echo ""

    # OpenAI API Key
    log_info "=== OpenAI API Key ==="
    echo "Get your API key from: https://platform.openai.com/api-keys"
    llm_api_key=$(prompt_secret "llm-api-key" "Enter your OpenAI API Key:")
    create_or_update_secret "vy-llm-api-key" "$llm_api_key" "OpenAI API key for LLM functionality"

    # Google API Key
    log_info "=== Google API Key ==="
    echo "Get your API key from: https://console.cloud.google.com/apis/credentials"
    echo "Make sure to enable the Custom Search API for this key"
    google_api_key=$(prompt_secret "google-api-key" "Enter your Google API Key:")
    create_or_update_secret "vy-google-api-key" "$google_api_key" "Google API key for search functionality"

    # Google Search Engine ID
    log_info "=== Google Custom Search Engine ID ==="
    echo "Create a custom search engine at: https://cse.google.com/cse/"
    echo "Get the Search Engine ID from your custom search engine settings"
    search_engine_id=$(prompt_secret "google-search-engine-id" "Enter your Google Custom Search Engine ID:")
    create_or_update_secret "vy-google-search-engine-id" "$search_engine_id" "Google Custom Search Engine ID"

    # Optional: Qdrant API Key (for cloud Qdrant)
    echo ""
    read -p "Do you want to configure Qdrant Cloud API key? (y/n): " configure_qdrant
    if [[ $configure_qdrant =~ ^[Yy]$ ]]; then
        log_info "=== Qdrant Cloud API Key ==="
        echo "Get your API key from your Qdrant Cloud dashboard"
        qdrant_api_key=$(prompt_secret "qdrant-api-key" "Enter your Qdrant Cloud API Key:")
        create_or_update_secret "vy-qdrant-api-key" "$qdrant_api_key" "Qdrant Cloud API key"
    fi
}

# Function to verify secrets
verify_secrets() {
    log_info "Verifying created secrets..."

    local required_secrets=("vy-llm-api-key" "vy-google-api-key" "vy-google-search-engine-id")
    local missing_secrets=()

    for secret in "${required_secrets[@]}"; do
        if ! gcloud secrets describe "$secret" --quiet &>/dev/null; then
            missing_secrets+=("$secret")
        else
            log_success "✓ $secret"
        fi
    done

    if [ ${#missing_secrets[@]} -gt 0 ]; then
        log_error "Missing required secrets:"
        for secret in "${missing_secrets[@]}"; do
            echo "  - $secret"
        done
        exit 1
    fi

    log_success "All required secrets are configured"
}

# Function to show secret usage in Cloud Run
show_usage() {
    echo ""
    log_info "Secrets have been created successfully!"
    echo ""
    echo "When deploying to Cloud Run, these secrets will be mounted as environment variables:"
    echo ""
    echo "Environment Variable          -> Secret"
    echo "VY_LLM_API_KEY               -> vy-llm-api-key:latest"
    echo "VY_GOOGLE_API_KEY            -> vy-google-api-key:latest"
    echo "VY_GOOGLE_SEARCH_ENGINE_ID   -> vy-google-search-engine-id:latest"

    if gcloud secrets describe "vy-qdrant-api-key" --quiet &>/dev/null; then
        echo "VY_QDRANT_API_KEY            -> vy-qdrant-api-key:latest"
    fi

    echo ""
    echo "The deployment script (deploy.sh) will automatically configure these."
    echo ""
    echo "Useful commands:"
    echo "  List secrets:    gcloud secrets list --filter='name~vy-'"
    echo "  View secret:     gcloud secrets versions access latest --secret=SECRET_NAME"
    echo "  Update secret:   echo 'NEW_VALUE' | gcloud secrets versions add SECRET_NAME --data-file=-"
    echo "  Delete secret:   gcloud secrets delete SECRET_NAME"
}

# Function to list existing secrets
list_secrets() {
    log_info "Current Vy-related secrets:"
    gcloud secrets list --filter='name~vy-' --format='table(name,createTime,labels.app)'
}

# Main function
main() {
    echo ""
    log_info "Vy Web - Google Cloud Secrets Setup"
    echo ""

    check_gcloud
    enable_apis

    case "${1:-setup}" in
        "setup"|"")
            setup_secrets
            verify_secrets
            show_usage
            ;;
        "list")
            list_secrets
            ;;
        "verify")
            verify_secrets
            ;;
        *)
            echo "Usage: $0 [setup|list|verify]"
            echo ""
            echo "Commands:"
            echo "  setup   - Setup all required secrets (default)"
            echo "  list    - List existing Vy-related secrets"
            echo "  verify  - Verify all required secrets exist"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
