# ✅ PatentRAG Deployment Checklist

## Before You Start

- [ ] **OpenAI API Key**: Get one from [platform.openai.com](https://platform.openai.com)
- [ ] **GitHub Account**: Make sure your code is pushed to GitHub
- [ ] **Public Repository**: Free tiers require public repos

## 🚀 Railway Deployment (Recommended)

### Step 1: Sign Up
- [ ] Go to [railway.app](https://railway.app)
- [ ] Sign up with GitHub
- [ ] Connect your repository

### Step 2: Add Databases
- [ ] Click "New Service" → "Database" → "PostgreSQL"
- [ ] Click "New Service" → "Database" → "Redis"
- [ ] Copy the connection URLs (you'll need these)

### Step 3: Deploy Backend
- [ ] Click "New Service" → "GitHub Repo"
- [ ] Select your repository
- [ ] Railway will auto-detect it's a Rust project
- [ ] Add environment variables:
  - [ ] `DATABASE_URL` = (from PostgreSQL service)
  - [ ] `REDIS_URL` = (from Redis service)
  - [ ] `OPENAI_API_KEY` = (your OpenAI key)

### Step 4: Deploy Frontend
- [ ] Click "New Service" → "GitHub Repo" again
- [ ] Select the same repository
- [ ] Railway will auto-detect it's a Next.js project
- [ ] Add environment variable:
  - [ ] `NEXT_PUBLIC_API_URL` = (your backend URL)

### Step 5: Test
- [ ] Visit your frontend URL
- [ ] Upload a PDF
- [ ] Try asking a question
- [ ] Check if chat history works

## 🎉 You're Done!

Your PatentRAG app is now live and accessible from anywhere!

**Frontend URL**: `https://your-app-name.railway.app`
**Backend URL**: `https://your-backend-name.railway.app`

## 🆘 Need Help?

If something goes wrong:
1. Check the logs in Railway dashboard
2. Verify all environment variables are set
3. Make sure your code compiles locally first
4. Check that databases are running

## 💡 Pro Tips

- **Free Tier Limits**: Railway gives you $5/month free credit
- **Custom Domain**: You can add a custom domain later
- **Monitoring**: Check the Railway dashboard for usage stats
- **Updates**: Push to GitHub to automatically redeploy 