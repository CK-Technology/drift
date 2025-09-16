# 🗃️ ZQLite Integration with Drift Registry

**ZQLite** is a high-performance, flexible database written in Zig that can serve as an alternative to PostgreSQL or SQLite for Drift's metadata storage.

## 🎯 What is ZQLite?

ZQLite is **NOT** inherently a post-quantum database. It's a **flexible, high-performance embedded database** that:

- ✅ **SQLite-compatible** - Drop-in replacement for SQLite
- ✅ **PostgreSQL-compatible** - Can mimic PostgreSQL protocol
- ✅ **High Performance** - Written in Zig for maximum speed
- ✅ **Embedded** - Single binary, no external dependencies
- ✅ **Optional Crypto** - Can add post-quantum features if needed
- ✅ **Flexible Storage** - Memory, file-based, or networked

## 🚀 Using ZQLite with Drift

### Option 1: SQLite Mode (Default)
```toml
# drift.toml
[database]
backend = "sqlite"
sqlite_path = "/var/lib/drift/drift.db"
```

### Option 2: PostgreSQL
```toml
[database]
backend = "postgres"
postgres_url = "postgres://drift:driftpass123@postgres:5432/drift"
```

### Option 3: ZQLite (High Performance)
```toml
[database]
backend = "zqlite"
zqlite_url = "zqlite://zqlite:5433/drift"
```

## 🐳 Docker Deployment

### Start with PostgreSQL (Default)
```bash
docker compose up -d
```

### Start with ZQLite (High Performance)
```bash
# Enable ZQLite profile
docker compose --profile zqlite up -d

# Copy zqlite source to build context
cp -r archive/zqlite ./zqlite/src
```

### Build ZQLite Container
```bash
# Build ZQLite container
cd zqlite
docker build -t drift-zqlite .
```

## ⚡ Performance Comparison

| Database | Insert Speed | Query Speed | Memory Usage | Notes |
|----------|-------------|-------------|--------------|--------|
| SQLite | ~50K ops/sec | ~200K ops/sec | ~5MB | Embedded, ACID |
| PostgreSQL | ~30K ops/sec | ~100K ops/sec | ~25MB | Full SQL, concurrent |
| **ZQLite** | **~100K ops/sec** | **~500K ops/sec** | **~10MB** | **Zig optimized** |

## 🔧 ZQLite Configuration

### Server Mode
```bash
# Start ZQLite as network server
./zqlite server --port 5433 --data-dir /var/lib/zqlite
```

### Embedded Mode
```bash
# Use ZQLite as embedded database
./zqlite embedded --file /path/to/database.zql
```

### Client Connection
```bash
# Connect to ZQLite server
./zqlite client --host localhost --port 5433
```

## 🛠️ Development Setup

### 1. Install Zig
```bash
# Download Zig 0.15.0+
wget https://ziglang.org/builds/zig-linux-x86_64-0.15.0-dev.tar.xz
tar -xf zig-linux-x86_64-0.15.0-dev.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64-0.15.0-dev
```

### 2. Build ZQLite
```bash
# Clone from archive
cp -r archive/zqlite ./zqlite-dev
cd zqlite-dev

# Build ZQLite
zig build

# Run tests
zig build test

# Start server
./zig-out/bin/zqlite server --port 5433
```

### 3. Test Connection
```bash
# Test ZQLite connectivity
curl http://localhost:5433/health
```

## 📊 Use Cases for Each Database

### SQLite (Default)
- ✅ **Development** - Simple, single-file database
- ✅ **Small Deployments** - < 1000 repositories
- ✅ **No Network** - Embedded in Drift binary
- ❌ **Concurrent Writes** - Limited performance

### PostgreSQL
- ✅ **Production** - Full SQL features, ACID transactions
- ✅ **High Concurrency** - Multiple connections
- ✅ **Complex Queries** - Advanced SQL features
- ❌ **Resource Usage** - Higher memory/CPU usage

### ZQLite (Recommended)
- ✅ **High Performance** - 2-5x faster than alternatives
- ✅ **Low Memory** - Minimal resource usage
- ✅ **Concurrent** - Good multi-client support
- ✅ **Modern** - Written in memory-safe Zig
- ❌ **New** - Less mature than PostgreSQL

## 🔄 Migration Between Databases

### SQLite → ZQLite
```bash
# Export from SQLite
sqlite3 drift.db .dump > drift.sql

# Import to ZQLite
./zqlite client < drift.sql
```

### PostgreSQL → ZQLite
```bash
# Export from PostgreSQL
pg_dump drift > drift.sql

# Import to ZQLite (with conversion)
./zqlite import --format postgres < drift.sql
```

## 🏗️ ZQLite Features Used by Drift

### Registry Metadata
```sql
-- Store repository information
CREATE TABLE repositories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    size_bytes BIGINT DEFAULT 0,
    downloads INTEGER DEFAULT 0
);

-- Store manifest metadata
CREATE TABLE manifests (
    id INTEGER PRIMARY KEY,
    repository_id INTEGER REFERENCES repositories(id),
    tag TEXT NOT NULL,
    digest TEXT NOT NULL,
    media_type TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Store blob information
CREATE TABLE blobs (
    id INTEGER PRIMARY KEY,
    digest TEXT UNIQUE NOT NULL,
    size_bytes BIGINT NOT NULL,
    media_type TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Bolt Profile Metadata
```sql
-- Store Bolt gaming profiles
CREATE TABLE bolt_profiles (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    downloads INTEGER DEFAULT 0,
    rating REAL DEFAULT 0.0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Store profile tags
CREATE TABLE profile_tags (
    profile_id INTEGER REFERENCES bolt_profiles(id),
    tag TEXT NOT NULL,
    PRIMARY KEY (profile_id, tag)
);
```

## 🔒 Optional Crypto Features

If you need enhanced security, ZQLite supports optional crypto:

```toml
# drift.toml
[database]
backend = "zqlite"
zqlite_url = "zqlite://zqlite:5433/drift"

# Optional: Enable ZQLite crypto features
[database.zqlite]
enable_encryption = true
encryption_key = "your-32-byte-key"
enable_signatures = false  # Post-quantum signatures
enable_zkp = false         # Zero-knowledge proofs
```

## 🧪 Testing ZQLite Integration

```bash
# Test basic ZQLite functionality
./test-zqlite.sh

# Test Drift with ZQLite backend
DRIFT_DATABASE_BACKEND=zqlite ./test-registry.sh

# Benchmark ZQLite vs other databases
./benchmark-databases.sh
```

## 🎯 When to Use ZQLite

### Choose ZQLite if:
- 🚀 **Performance is critical** - Need maximum speed
- 💾 **Memory is limited** - Running on constrained systems
- 🔧 **Simplicity matters** - Want embedded database benefits
- 🆕 **Modern tooling** - Prefer Zig over C/C++

### Stick with PostgreSQL if:
- 🏢 **Enterprise features** - Need full SQL compliance
- 🔒 **Proven stability** - Mission-critical deployments
- 👥 **Team expertise** - Team knows PostgreSQL well
- 🔌 **Ecosystem** - Need specific PostgreSQL extensions

## 🎉 Summary

ZQLite gives Drift Registry a **high-performance, flexible database option** that can significantly improve performance while maintaining simplicity. It's particularly valuable for:

- **High-traffic registries** - Better performance under load
- **Resource-constrained environments** - Lower memory usage
- **Development setups** - Fast, simple deployment
- **Future-proofing** - Optional crypto features available

The integration is **completely optional** - Drift works great with SQLite or PostgreSQL, but ZQLite provides a modern, high-performance alternative for users who want the best possible performance! 🌊