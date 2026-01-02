# SkyPulseDB

**A high-performance time-series database built in Rust, purpose-designed for weather and meteorological data.**

---

## Overview

SkyPulseDB is a specialized time-series database engineered from the ground up to handle the unique demands of weather observation data. Unlike general-purpose databases that treat all time-series data equally, SkyPulseDB leverages the inherent patterns in meteorological data—predictable sampling intervals, correlated sensor readings, and bounded value ranges—to achieve superior compression ratios and query performance.

Built entirely in Rust, SkyPulseDB prioritizes memory safety, zero-copy operations, and predictable latency, making it ideal for both edge deployments at weather stations and centralized data centers processing thousands of observation points.

---

## Key Features

### Weather-Optimized Storage

SkyPulseDB employs a wide-table schema design where each observation record contains all sensor readings from a single timestamp. This approach eliminates the need for expensive JOIN operations and enables columnar compression algorithms tuned for specific weather elements—Gorilla compression for temperature fluctuations, delta-of-delta encoding for timestamps, and specialized quantization for wind direction.

### Columnar Compression

Each weather element is stored and compressed independently, exploiting the statistical properties of meteorological data. Temperature readings from the same station rarely vary by more than a few degrees between consecutive observations; SkyPulseDB's XOR-based compression reduces these to just a few bits per value. Typical compression ratios range from 10:1 to 20:1 compared to raw storage.

### Time-Partitioned Architecture

Data is automatically partitioned into daily chunks, enabling efficient time-range queries and simplified data lifecycle management. Older chunks can be compressed more aggressively, archived to cold storage, or purged according to configurable retention policies—all without impacting write performance for current data.

### High-Throughput Ingestion

The write path is optimized for sustained high-volume ingestion typical of weather monitoring networks. An in-memory buffer absorbs bursts while a write-ahead log ensures durability. Background flush operations convert buffered data into compressed, indexed chunks without blocking incoming writes.

### SQL Query Interface

SkyPulseDB supports a familiar SQL dialect with extensions for time-series operations. Built-in functions like `time_bucket()`, `first()`, `last()`, and interpolation operators make common meteorological queries straightforward to express.

---

## Use Cases

- **National Weather Services**: Centralized storage for nationwide observation networks with thousands of automated weather stations reporting at minute-level intervals.

- **Research Institutions**: Long-term climate data archives requiring efficient storage and fast analytical queries across decades of historical observations.

- **Renewable Energy**: Wind and solar resource assessment requiring high-frequency sensor data with rapid aggregation for power output modeling.

- **Aviation Weather**: Real-time METAR/SPECI processing with low-latency access for flight planning and safety systems.

- **Agricultural Monitoring**: Field-level microclimate tracking for precision agriculture and frost warning systems.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     API Layer                           │
│            (gRPC / HTTP REST / Arrow Flight)            │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                    Query Engine                         │
│         (SQL Parser → Planner → Optimizer)              │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                   Storage Engine                        │
│  ┌──────────┐  ┌──────────┐  ┌────────────────────────┐ │
│  │ MemTable │  │   WAL    │  │   Chunk Store          │ │
│  │ (Write   │  │ (Dura-   │  │   (Compressed          │ │
│  │  Buffer) │  │  bility) │  │    Columnar Files)     │ │
│  └──────────┘  └──────────┘  └────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              Compression Layer                          │
│   (Gorilla / Delta-Delta / ZSTD / Custom Quantization)  │
└─────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/skypulsedb.git
cd skypulsedb

# Build release binary
cargo build --release

# Run the server
./target/release/skypulsedb --data-dir /var/lib/skypulsedb
```

### Ingesting Data

```bash
# Insert observations via HTTP API
curl -X POST http://localhost:8080/api/v1/write \
  -H "Content-Type: application/json" \
  -d '{
    "station_id": "TPE001",
    "time": "2025-01-02T10:00:00Z",
    "temp": 18.5,
    "humidity": 72,
    "pressure": 1013.2,
    "wind_speed": 3.4,
    "wind_dir": 270
  }'
```

### Querying Data

```sql
-- Recent observations from a station
SELECT time, temp, humidity, pressure
FROM observations
WHERE station_id = 'TPE001'
  AND time > now() - interval '24 hours'
ORDER BY time DESC;

-- Hourly averages across all stations
SELECT 
    time_bucket('1 hour', time) AS hour,
    station_id,
    avg(temp) AS temp_avg,
    max(wind_speed) AS wind_max,
    sum(precip) AS precip_total
FROM observations
WHERE time > now() - interval '7 days'
GROUP BY hour, station_id
ORDER BY hour DESC;

-- Find extreme temperatures
SELECT station_id, time, temp
FROM observations
WHERE time > now() - interval '30 days'
  AND temp = (SELECT max(temp) FROM observations WHERE time > now() - interval '30 days');
```

---

## Performance

Benchmarks conducted on a single node (AMD Ryzen 9, 64GB RAM, NVMe SSD):

| Metric | Performance |
|--------|-------------|
| Write throughput | 850,000 points/sec |
| Query latency (single station, 24h) | < 5 ms |
| Query latency (100 stations, 24h) | < 50 ms |
| Aggregation (1 year, hourly buckets) | < 200 ms |
| Compression ratio | 12:1 typical |
| Storage (100 stations, 1-min interval, 1 year) | ~800 MB |

---

## Roadmap

- [x] Core storage engine with WAL and chunk management
- [x] Gorilla compression for floating-point values
- [x] Basic SQL query support
- [ ] Continuous aggregates (automatic rollups)
- [ ] Cluster mode with horizontal scaling
- [ ] Grafana data source plugin
- [ ] S3-compatible cold storage tiering
- [ ] Geospatial indexing for spatial queries
- [ ] Anomaly detection operators

---

## Contributing

SkyPulseDB is open source under the Apache 2.0 license. Contributions are welcome—whether it's bug fixes, new compression algorithms, or query optimizations.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

---

## License

Apache License 2.0

---

## Acknowledgments

SkyPulseDB draws inspiration from several excellent projects in the time-series database space, including TimescaleDB, InfluxDB, QuestDB, and the Gorilla paper from Facebook. We're grateful to the open-source community for paving the way.