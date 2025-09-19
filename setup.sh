#!/usr/bin/env bash
set -e

PROJECT_ROOT=$(pwd)

echo "Setting up project at $PROJECT_ROOT"

# ----------------------------
# 1️⃣ Backend (FastAPI)
# ----------------------------
echo "Setting up backend..."
mkdir -p $PROJECT_ROOT/backend/app
cd $PROJECT_ROOT/backend

cat > app/main.py <<EOL
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
def read_root():
    return {"message": "Hello from FastAPI!"}
EOL

# ----------------------------
# 2️⃣ Shared (JS/TS logic)
# ----------------------------
echo "Setting up shared code..."
mkdir -p $PROJECT_ROOT/frontend/shared/src
cd $PROJECT_ROOT/frontend/shared

npm init -y
npm install axios
npm install --save-dev typescript @types/node

# Add type=module to package.json
jq '. + {"type":"module"}' package.json > package.tmp.json && mv package.tmp.json package.json

# Create TS config compatible with ESM + Node
cat > tsconfig.json <<EOL
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Node",
    "rootDir": "src",
    "outDir": "dist",
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "strict": true,
    "skipLibCheck": true,
    "resolveJsonModule": true,
    "declaration": true
  },
  "include": ["src"]
}
EOL

# Create API and model files
cat > src/api.ts <<EOL
import axios from "axios";

const api = axios.create({
  baseURL: "http://127.0.0.1:8000",
});

export const getHello = async () => {
  const res = await api.get("/");
  return res.data;
};
EOL

cat > src/models.ts <<EOL
export interface HelloResponse {
  message: string;
}
EOL

# Build shared
npx tsc

# ----------------------------
# 3️⃣ Desktop (Tauri + React)
# ----------------------------
echo "Setting up desktop app..."

cd $PROJECT_ROOT/frontend
npm create tauri-app@latest desktop --template react-ts

cd desktop
npm install ../shared

# ----------------------------
# 4️⃣ Mobile (React Native + Expo)
# ----------------------------
echo "Setting up mobile app..."

cd $PROJECT_ROOT/frontend
npx create-expo-app mobile --template

cd mobile
npm install ../shared

echo "✅ Setup complete!"
echo "Backend: cd $PROJECT_ROOT/backend && fastapi dev"
echo "Desktop: cd $PROJECT_ROOT/frontend/desktop && npm run tauri dev"
echo "Mobile: cd $PROJECT_ROOT/frontend/mobile && npx expo start"
