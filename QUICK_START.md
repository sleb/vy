# Vy Web - Quick Start Deployment Guide

🚀 **Deploy Vy Web to Google Cloud Run in 5 minutes**

## Prerequisites Checklist

- [ ] Docker installed and running
- [ ] Google Cloud CLI installed (`gcloud`)
- [ ] Google Cloud Project with billing enabled
- [ ] OpenAI API key
- [ ] Google API key with Custom Search API enabled
- [ ] Google Custom Search Engine ID

## 🏃‍♂️ Quick Deploy (5 steps)

### 1. Clone & Navigate
```bash
cd vy
```

### 2. Configure Project
Edit `deploy.sh` and set your Google Cloud Project ID:
```bash
nano deploy.sh
# Set: PROJECT_ID="your-gcp-project-id"
```

### 3. Make Scripts Executable
```bash
chmod +x deploy.sh setup-secrets.sh
```

### 4. Setup Secrets (First Time Only)
```bash
./setup-secrets.sh
```
This will prompt you for:
- OpenAI API key
- Google API key
- Google Custom Search Engine ID

### 5. Deploy to Cloud Run
```bash
./deploy.sh
```

**That's it!** Your service will be deployed and accessible at the URL shown in the output.

## 🧪 Local Testing (Optional)

Test locally before deploying:

1. **Copy environment template**:
   ```bash
   cp .env.example .env
   ```

2. **Add your API keys to `.env`**:
   ```bash
   nano .env
   ```

3. **Start with Docker Compose**:
   ```bash
   docker-compose up --build
   ```

4. **Test the API**:
   ```bash
   curl http://localhost:3001/health
   curl -X POST http://localhost:3001/api/chat \
     -H "Content-Type: application/json" \
     -d '{"message": "Hello!"}'
   ```

## 📡 API Usage

Once deployed, use your service:

### Health Check
```bash
curl https://YOUR-SERVICE-URL/health
```

### Chat API
```bash
curl -X POST https://YOUR-SERVICE-URL/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is the weather like today?",
    "conversation_id": "optional-uuid"
  }'
```

## 🔧 Common Commands

```bash
# View deployment logs
gcloud logs tail --follow --log-filter='resource.type=cloud_run_revision AND resource.labels.service_name=vy-web'

# Update environment variables
gcloud run services update vy-web --region=us-central1 --set-env-vars=VY_LLM_MODEL_ID=gpt-4o

# Scale service
gcloud run services update vy-web --region=us-central1 --min-instances=1 --max-instances=5

# Delete service
gcloud run services delete vy-web --region=us-central1
```

## 🆘 Troubleshooting

**Build fails?**
```bash
docker system prune -a
./deploy.sh build
```

**Secrets missing?**
```bash
./setup-secrets.sh verify
gcloud secrets list --filter='name~vy-'
```

**Service errors?**
```bash
gcloud logs tail --follow --log-filter='resource.type=cloud_run_revision AND resource.labels.service_name=vy-web'
```

## 🔑 Required API Keys

1. **OpenAI API Key**: Get from [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
2. **Google API Key**: Get from [console.cloud.google.com/apis/credentials](https://console.cloud.google.com/apis/credentials)
3. **Google Search Engine ID**: Create at [cse.google.com](https://cse.google.com/cse/)

## 💰 Estimated Costs

- **Development/Testing**: $0-10/month
- **Light Production**: $10-50/month
- **Heavy Usage**: $50-200/month

*Costs include Cloud Run + OpenAI API + Google Search API usage*

---

**Need more details?** See `DOCKER_DEPLOYMENT.md` for comprehensive documentation.
