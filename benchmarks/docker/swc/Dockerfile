FROM node:18-alpine

# Install system dependencies
RUN apk add --no-cache \
    build-base \
    python3 \
    curl \
    git \
    bash

# Set working directory  
WORKDIR /workspace

# Install SWC and dependencies
RUN npm install -g \
    @swc/cli \
    @swc/core \
    @swc/helpers \
    typescript \
    source-map-support

# Create package.json for workspace
COPY package.json .
RUN npm install

# Copy configuration files
COPY swc.config.js .
COPY tsconfig.json .

# Install monitoring tools
RUN npm install -g clinic

# Create results directory
RUN mkdir -p /results/swc

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD node -e "console.log('SWC container healthy')" || exit 1

# Default command
CMD ["node", "--version"]