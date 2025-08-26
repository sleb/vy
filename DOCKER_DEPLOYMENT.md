# Vy Web - Docker & Google Cloud Run Deployment Guide

This guide walks you through dockerizing and deploying the `vy-web` service to Google Cloud Run.

## Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and running
- [Google Cloud CLI](https://cloud.google.com/sdk/docs/install) installed and configured
- A Google Cloud Project with billing enabled
- Required API keys (OpenAI, Google API, Google Custom Search)

### One-Command Deployment

1. **Edit the deployment script** with your project ID:
   ```bash
   # Edit deploy.sh and set your PROJECT_ID
   nano deploy.sh
   ```

2. **Make scripts executable**:
   ```bash
   chmod +x deploy.sh setup-secrets.sh
   ```

3. **Setup secrets** (first time only):
   ```bash
   ./setup-secrets.sh
   ```

4. **Deploy to Cloud Run**:
   ```bash
   ./deploy.sh
   ```

## Detailed Setup

### 1. API Keys Setup

You'll need these API keys:

#### OpenAI API Key
- Go to [OpenAI Platform](https://platform.openai.com/api-keys)
- Create a new API key
- Copy the key (starts with `sk-`)

#### Google API Key
- Go to [Google Cloud Console - Credentials](https://console.cloud.google.com/apis/credentials)
- Create a new API key
- Enable the "Custom Search API" for this key

#### Google Custom Search Engine ID
- Go to [Google Custom Search](https://cse.google.com/cse/)
- Create a new search engine
- Get the Search Engine ID from the settings

### 2. Local Development Setup

#### Using Docker Compose (Recommended)

1. **Copy environment template**:
   ```bash
   cp .env.example .env
   ```

2. **Edit `.env`** with your actual API keys:
   ```bash
   nano .env
   ```

3. **Start the services**:
   ```bash
   docker-compose up --build
   ```

4. **Test the service**:
   ```bash
   # Health check
   curl http://localhost:3001/health

   # Chat API
   curl -X POST http://localhost:3001/api/chat \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello, world!"}'
   ```

#### Using Docker Only

1. **Build the image**:
   ```bash
   docker build -t vy-web .
   ```

2. **Run the container**:
   ```bash
   docker run -p 3001:8080 \
     -e VY_LLM_API_KEY="your-openai-key" \
     -e VY_GOOGLE_API_KEY="your-google-key" \
     -e VY_GOOGLE_SEARCH_ENGINE_ID="your-search-engine-id" \
     vy-web
   ```

### 3. Google Cloud Run Deployment

#### Automated Deployment

Use the provided deployment script for the easiest experience:

1. **Configure your project**:
   ```bash
   # Edit deploy.sh and set PROJECT_ID
   PROJECT_ID="your-gcp-project-id"
   ```

2. **Setup secrets** (one-time):
   ```bash
   ./setup-secrets.sh
   ```

3. **Deploy**:
   ```bash
   ./deploy.sh
   ```

#### Manual Deployment

If you prefer manual control:

1. **Set up Google Cloud**:
   ```bash
   gcloud config set project YOUR_PROJECT_ID
   gcloud services enable cloudbuild.googleapis.com run.googleapis.com
   gcloud auth configure-docker
   ```

2. **Create secrets**:
   ```bash
   echo "your-openai-key" | gcloud secrets create vy-llm-api-key --data-file=-
   echo "your-google-key" | gcloud secrets create vy-google-api-key --data-file=-
   echo "your-search-engine-id" | gcloud secrets create vy-google-search-engine-id --data-file=-
   ```

3. **Build and push image**:
   ```bash
   docker build -t gcr.io/YOUR_PROJECT_ID/vy-web .
   docker push gcr.io/YOUR_PROJECT_ID/vy-web
   ```

4. **Deploy to Cloud Run**:
   ```bash
   gcloud run deploy vy-web \
     --image gcr.io/YOUR_PROJECT_ID/vy-web \
     --platform managed \
     --region us-central1 \
     --allow-unauthenticated \
     --memory 512Mi \
     --set-secrets "VY_LLM_API_KEY=vy-llm-api-key:latest,VY_GOOGLE_API_KEY=vy-google-api-key:latest,VY_GOOGLE_SEARCH_ENGINE_ID=vy-google-search-engine-id:latest"
   ```

## Configuration

### Environment Variables

The service can be configured with these environment variables:

#### Required
- `VY_LLM_API_KEY` - OpenAI API key
- `VY_GOOGLE_API_KEY` - Google API key
- `VY_GOOGLE_SEARCH_ENGINE_ID` - Google Custom Search Engine ID

#### Optional (with defaults)
- `VY_LLM_MODEL_ID` - Default: `gpt-4o-mini`
- `VY_MEMORY_MODEL_ID` - Default: `gpt-4o-mini`
- `VY_MEMORY_SIMILARITY_MODEL_ID` - Default: `gpt-4o-mini`
- `VY_EMBEDDING_MODEL` - Default: `text-embedding-3-small`
- `VY_DEFAULT_CHAT_MODE` - Default: `web`
- `VY_QDRANT_URL` - Default: `http://localhost:6333`
- `VY_COLLECTION_NAME` - Default: `vy_memories`
- `VY_VECTOR_MEMORY_OPENAI_API_KEY` - Defaults to `VY_LLM_API_KEY`

#### Server Configuration
- `HOST` - Default: `0.0.0.0`
- `PORT` - Default: `8080` (Cloud Run will set this)

### Cloud Run Specific Settings

The deployment is optimized for Cloud Run with:

- **Memory**: 512Mi (configurable)
- **CPU**: 1 vCPU (configurable)
- **Concurrency**: 100 requests per instance
- **Auto-scaling**: 0-10 instances
- **Timeout**: 300 seconds
- **Health checks**: Enabled on `/health`

## API Endpoints

### Health Check
```bash
GET /health
```
Returns service health status and version.

### Chat API
```bash
POST /api/chat
Content-Type: application/json

{
  "message": "Your message here",
  "conversation_id": "optional-uuid"
}
```

Response:
```json
{
  "response": "AI response here",
  "conversation_id": "uuid"
}
```

## Troubleshooting

### Common Issues

#### Build Failures

1. **Rust compilation errors**:
   ```bash
   # Clean and rebuild
   docker system prune -a
   docker build --no-cache -t vy-web .
   ```

2. **SSL/TLS certificate errors**:
   - Make sure `ca-certificates` is installed in the Docker image
   - Check your network connection and proxy settings

#### Runtime Issues

1. **Service won't start**:
   ```bash
   # Check logs
   gcloud logs tail --follow --log-filter='resource.type=cloud_run_revision AND resource.labels.service_name=vy-web'
   ```

2. **API key errors**:
   ```bash
   # Verify secrets exist
   gcloud secrets list --filter='name~vy-'

   # Check secret values (careful - this shows the actual secret!)
   gcloud secrets versions access latest --secret=vy-llm-api-key
   ```

3. **Memory or timeout issues**:
   ```bash
   # Increase memory allocation
   gcloud run services update vy-web --memory=1Gi --region=us-central1

   # Increase timeout
   gcloud run services update vy-web --timeout=600 --region=us-central1
   ```

#### Network Issues

1. **Can't connect to external APIs**:
   - Verify your API keys are valid
   - Check if your Cloud Run service has internet access
   - Verify API quotas and billing

2. **Vector database connection**:
   - For local Qdrant: make sure it's running and accessible
   - For Qdrant Cloud: verify URL and API key

### Debugging Commands

```bash
# View service details
gcloud run services describe vy-web --region=us-central1

# Follow logs in real-time
gcloud logs tail --follow --log-filter='resource.type=cloud_run_revision AND resource.labels.service_name=vy-web'

# Test locally with same image
docker run -it --rm \
  -e VY_LLM_API_KEY="$VY_LLM_API_KEY" \
  -e VY_GOOGLE_API_KEY="$VY_GOOGLE_API_KEY" \
  -e VY_GOOGLE_SEARCH_ENGINE_ID="$VY_GOOGLE_SEARCH_ENGINE_ID" \
  -p 8080:8080 \
  vy-web

# Shell into running container (for debugging)
docker exec -it container_id /bin/bash
```

## Security Best Practices

1. **Use secrets management**: Never hardcode API keys in images or code
2. **Rotate keys regularly**: Update secrets periodically
3. **Limit permissions**: Use IAM to restrict service account permissions
4. **Enable audit logging**: Track who accesses your secrets
5. **Use private networking**: Consider VPC for sensitive workloads

## Cost Optimization

### Cloud Run Pricing Tips

1. **Right-size resources**: Start with minimal CPU/memory and scale up if needed
2. **Use min-instances=0**: Let it scale to zero when not in use
3. **Set appropriate concurrency**: Higher concurrency = fewer instances
4. **Monitor usage**: Use Cloud Monitoring to track costs

### Estimated Costs (USD, approximate)

- **Minimal usage** (few requests/day): $0-5/month
- **Development** (moderate testing): $5-20/month
- **Production** (regular usage): $20-100/month

*Costs depend on request volume, response times, and external API usage (OpenAI, Google).*

## Monitoring and Observability

### Health Monitoring

The service includes comprehensive health checks:

```bash
# Basic health check
curl https://your-service-url/health

# Detailed monitoring
gcloud monitoring dashboards create --config-from-file=monitoring-dashboard.json
```

### Logging

Structured logging is enabled by default:

```bash
# View recent logs
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=vy-web" --limit=50

# Filter by severity
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=vy-web AND severity>=ERROR"
```

## Scaling and Performance

### Auto-scaling Configuration

The service is configured for optimal auto-scaling:

- **Cold starts**: ~2-5 seconds
- **Concurrent requests**: Up to 100 per instance
- **Scale-up time**: ~5-10 seconds
- **Scale-down time**: ~15 minutes (Cloud Run default)

### Performance Tuning

1. **Optimize cold starts**:
   ```bash
   gcloud run services update vy-web --min-instances=1 --region=us-central1
   ```

2. **Increase concurrency** for CPU-light workloads:
   ```bash
   gcloud run services update vy-web --concurrency=200 --region=us-central1
   ```

3. **Add more CPU** for CPU-intensive tasks:
   ```bash
   gcloud run services update vy-web --cpu=2 --region=us-central1
   ```

## Contributing

When making changes to the Docker configuration:

1. Test locally first with `docker-compose up`
2. Update this documentation if needed
3. Test deployment to a development Cloud Run service
4. Update version tags appropriately

## Support

For issues related to:
- **Docker setup**: Check Docker documentation and this guide
- **Google Cloud**: Check Cloud Run documentation and error logs
- **Vy application**: Check the main project README and issues
- **API integrations**: Verify API keys and check service status pages

---

**Last updated**: December 2024
**Docker version**: 24.0+
**Cloud Run**: Generation 2
**Rust version**: 1.75+
