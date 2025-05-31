# Copy the React source code
cp -r src ../ordinator-api/static_files/scheduler/

# Copy the public folder (static assets)
cp -r public ../ordinator-api/static_files/scheduler/

# Copy the compiled static files (if already built)
cp -r dist ../ordinator-api/static_files/scheduler/

# Copy essential project configuration files
cp package.json ../ordinator-api/static_files/scheduler/
cp package-lock.json ../ordinator-api/static_files/scheduler/
cp vite.config.ts ../ordinator-api/static_files/scheduler/
cp tsconfig.json ../ordinator-api/static_files/scheduler/
cp tsconfig.app.json ../ordinator-api/static_files/scheduler/
cp tsconfig.node.json ../ordinator-api/static_files/scheduler/
cp tailwind.config.js ../ordinator-api/static_files/scheduler/
cp postcss.config.js ../ordinator-api/static_files/scheduler/

# Copy the main HTML file
cp index.html ../ordinator-api/static_files/scheduler/

# Optional: Copy the TypeScript build info (if needed)
cp tsconfig.node.tsbuildinfo ../ordinator-api/static_files/scheduler/
cp tsconfig.app.tsbuildinfo ../ordinator-api/static_files/scheduler/
