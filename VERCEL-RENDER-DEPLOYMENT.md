# 🚀 Vercel + Render Deployment (Easy & Reliable!)

## Step 1: Deploy Frontend to Vercel (5 minutes)

1. **Go to [vercel.com](https://vercel.com)**
2. **Sign up with GitHub**
3. **Click "New Project"**
4. **Import your GitHub repository**
5. **Vercel automatically detects Next.js and deploys**
6. **Copy your Vercel URL** (you'll need this for the backend)

**That's it!** Vercel handles everything automatically.

## Step 2: Deploy Backend to Render (10 minutes)

1. **Go to [render.com](https://render.com)**
2. **Sign up with GitHub**
3. **Click "New" → "Web Service"**
4. **Connect your GitHub repository**
5. **Configure:**
   - **Name:** `patentrag-backend`
   - **Environment:** `Rust`
   - **Build Command:** `cargo build --release`
   - **Start Command:** `./target/release/backend`
6. **Click "Create Web Service"**

## Step 3: Add PostgreSQL Database

1. **In Render dashboard, click "New" → "PostgreSQL"**
2. **Name:** `patentrag-postgres`
3. **Click "Create Database"**
4. **Copy the connection string**

## Step 4: Add Redis Database

1. **Click "New" → "Redis"**
2. **Name:** `patentrag-redis`
3. **Click "Create Redis"**
4. **Copy the connection string**

## Step 5: Configure Environment Variables

1. **Go back to your backend service**
2. **Click "Environment" tab**
3. **Add these variables:**
   ```
   DATABASE_URL=postgresql://... (from PostgreSQL)
   REDIS_URL=redis://... (from Redis)
   OPENAI_API_KEY=your_openai_api_key_here
   ```

## Step 6: Connect Frontend to Backend

1. **Go to your Vercel project**
2. **Click "Settings" → "Environment Variables"**
3. **Add:**
   ```
   NEXT_PUBLIC_API_URL=https://your-render-backend-url.onrender.com
   ```
4. **Redeploy** (Vercel will do this automatically)

## Step 7: Test Your App

1. **Visit your Vercel URL**
2. **Upload a PDF**
3. **Try asking a question**
4. **Everything should work!**

## 🎉 Done!

- **Frontend:** `https://your-app.vercel.app`
- **Backend:** `https://your-backend.onrender.com`

## 💡 Why This is Better Than Railway

- ✅ **Vercel:** Best Next.js hosting, super reliable
- ✅ **Render:** Simple, good free tier, reliable
- ✅ **No complex configuration needed**
- ✅ **Automatic deployments**
- ✅ **Better error messages**
- ✅ **More stable**

## 🆘 Need Help?

If something fails:
1. **Check Render logs** in the dashboard
2. **Check Vercel logs** in the dashboard
3. **Make sure environment variables are set correctly**
4. **Verify your OpenAI API key works** 