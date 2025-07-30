FROM node:18-alpine

# Install system dependencies
RUN apk add --no-cache \
    build-base \
    python3 \
    curl \
    git \
    bash

WORKDIR /workspace

# Install TypeScript compiler and tools
RUN npm install -g \
    typescript \
    ts-node \
    @typescript-eslint/parser \
    @typescript-eslint/eslint-plugin

# Create package.json
COPY package.json .
RUN npm install

# Copy TypeScript configuration
COPY tsconfig.json .
COPY tsconfig.benchmark.json .

# Install profiling tools
RUN npm install -g clinic autocannon

# Create results directory
RUN mkdir -p /results/tsc

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD tsc --version || exit 1

CMD ["tsc", "--version"]