# 🚀 PatentRAG Deployment Guide (No Docker Required!)

## 🎯 Recommended: Railway (Easiest Option)

**Perfect for:** Beginners, no Docker knowledge needed, free tier available

### Step-by-Step Guide:

#### 1. **Sign Up & Connect**
1. Go to [railway.app](https://railway.app) and sign up
2. Connect your GitHub repository
3. Railway will automatically detect your Rust backend and Next.js frontend

#### 2. **Add Databases**
1. Click "New Service" → "Database" → "PostgreSQL"
2. Click "New Service" → "Database" → "Redis"
3. Note down the connection URLs (you'll need these)

#### 3. **Deploy Backend**
1. Click "New Service" → "GitHub Repo"
2. Select your repository
3. Railway will automatically detect it's a Rust project
4. Set environment variables:
   ```
   DATABASE_URL=postgresql://... (from PostgreSQL service)
   REDIS_URL=redis://... (from Redis service)
   OPENAI_API_KEY=your_openai_api_key_here
   ```

#### 4. **Deploy Frontend**
1. Click "New Service" → "GitHub Repo" again
2. Select the same repository
3. Railway will detect it's a Next.js project
4. Set environment variable:
   ```
   NEXT_PUBLIC_API_URL=https://your-backend-url.railway.app
   ```

#### 5. **Done!**
- Your app will be live at the frontend URL
- Backend API at the backend URL
- Databases automatically connected

---

## 🚀 Alternative: Vercel + Railway

**Best for:** Excellent Next.js hosting + backend services

### Frontend on Vercel:
1. Go to [vercel.com](https://vercel.com) and sign up
2. Click "New Project" → Import your GitHub repo
3. Vercel automatically detects Next.js and deploys
4. Set environment variable:
   ```
   NEXT_PUBLIC_API_URL=https://your-railway-backend-url.railway.app
   ```

### Backend on Railway:
- Follow steps 1-3 from the Railway guide above

---

## 🚀 Alternative: Render (Free Tier)

**Good for:** Free tier, simple setup

### Backend on Render:
1. Go to [render.com](https://render.com) and sign up
2. Click "New" → "Web Service"
3. Connect your GitHub repo
4. Configure:
   - **Name:** `patentrag-backend`
   - **Environment:** `Rust`
   - **Build Command:** `cargo build --release`
   - **Start Command:** `./target/release/backend`
5. Add environment variables (same as Railway)

### Frontend on Render:
1. Click "New" → "Static Site"
2. Connect your GitHub repo
3. Configure:
   - **Name:** `patentrag-frontend`
   - **Build Command:** `npm run build`
   - **Publish Directory:** `out`
4. Add environment variable:
   ```
   NEXT_PUBLIC_API_URL=https://your-render-backend-url.onrender.com
   ```

### Add Databases on Render:
1. Click "New" → "PostgreSQL"
2. Click "New" → "Redis"
3. Copy connection URLs to your services

---

## 🔧 Environment Variables

### Backend Variables:
```bash
DATABASE_URL=postgresql://user:password@host:5432/database
REDIS_URL=redis://host:6379
OPENAI_API_KEY=your_openai_api_key_here
```

### Frontend Variables:
```bash
NEXT_PUBLIC_API_URL=https://your-backend-url.com
```

---

## 📋 Pre-Deployment Checklist

- [ ] Get an OpenAI API key from [platform.openai.com](https://platform.openai.com)
- [ ] Push all your code to GitHub
- [ ] Make sure your repo is public (for free tiers)
- [ ] Have your database SQL files ready (in `data/` folder)

---

## 🚨 Common Issues & Solutions

### "Build Failed"
- Check that all dependencies are in `Cargo.toml` and `package.json`
- Ensure your code compiles locally first

### "Database Connection Error"
- Double-check your `DATABASE_URL` format
- Make sure the database service is running

### "API Not Found"
- Verify `NEXT_PUBLIC_API_URL` points to your backend
- Check that your backend is deployed and running

### "CORS Error"
- The backend already has CORS configured
- Make sure you're using the correct API URL

---

## 🎉 Success!

Once deployed, you'll have:
- ✅ Live web application
- ✅ Working PDF upload and processing
- ✅ AI-powered Q&A and search
- ✅ Chat history and management
- ✅ Smart summaries and related documents
- ✅ Export functionality

Your PatentRAG app will be accessible from anywhere in the world! 🌍 